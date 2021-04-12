use crate::document::Document;
use std::collections::HashMap;
use std::sync::Arc;
use volpe_parser::{
    lexeme_kind::LexemeKind,
    offset::Offset,
    syntax::{RuleKind, Syntax},
};

pub struct Variable {
    pub declaration: Offset,
    pub parameter: bool,
}

impl Document {
    pub fn variable_pass(&mut self) {
        if self.vars.is_some() {
            return;
        }

        fn recurse<'a>(
            syntax: Syntax<'a>,
            scoped_vars: &mut HashMap<&'a str, Arc<Variable>>,
            pos: &mut Offset,
            output_vars: &mut HashMap<Offset, Arc<Variable>>,
        ) {
            // Leaf node - Lexeme.
            if syntax.kind.is_none() {
                let lexeme = syntax.lexeme;
                // It is an identifier.
                if matches!(lexeme.kind, LexemeKind::Ident) {
                    // Track whether we made a new variable.
                    let mut new = false;

                    let name = &lexeme.string[..lexeme.token_length.char as usize];
                    if let Some(next) = lexeme.next.as_deref() {
                        // Check the next lexeme to see if it is an assignment or function.
                        let parameter = matches!(next.kind, LexemeKind::Func);
                        let assignment = matches!(next.kind, LexemeKind::Assign);
                        if parameter || assignment {
                            // New variable was created.
                            new = true;
                            let var = Arc::new(Variable {
                                declaration: *pos,
                                parameter,
                            });
                            scoped_vars.insert(name, Arc::clone(&var));
                            output_vars.insert(*pos, var);
                        }
                    }

                    if !new {
                        // Connect the use of a variable to its declaration.
                        if let Some(var) = scoped_vars.get(name) {
                            output_vars.insert(*pos, Arc::clone(var));
                        }
                    }
                }

                *pos += syntax.lexeme.length;
                return;
            }

            // Rule node
            // Optionally save the current scope if we are entering a new one.
            let mut checkpoint = None;
            if matches!(syntax.kind.unwrap(), RuleKind::Func | RuleKind::Expr) {
                checkpoint = Some(scoped_vars.clone());
            }

            // Parse the right hand side of assignment separately.
            // The variable we are just defining is not yet in scope for the rhs.
            let mut syntax_iter = syntax.into_iter();
            let assignment = syntax_iter
                .position(|child| matches!(child.lexeme.kind, LexemeKind::Assign));
            let semicolon = syntax_iter
                .position(|child| matches!(child.lexeme.kind, LexemeKind::Semicolon))
                .unwrap_or(usize::MAX);

            if let Some(start) = assignment {
                // TODO Maybe get rid of this clone by tracking pos instead?
                let mut before_assign = scoped_vars.clone();
                for (i, child) in syntax.into_iter().enumerate() {
                    if i < start || i - start > semicolon {
                        recurse(child, scoped_vars, pos, output_vars)
                    } else {
                        recurse(child, &mut before_assign, pos, output_vars)
                    }
                }

            // If it is not assignment we just recurse normally.
            } else {
                for child in syntax {
                    recurse(child, scoped_vars, pos, output_vars)
                }
            }

            // If we made a checkpoint it means we should use it.
            // This "drops" the local variables made in that scope.
            if let Some(before) = checkpoint {
                *scoped_vars = before;
            }
        }

        // Begin recursion.
        let mut output_vars = HashMap::new();
        let mut pos = Offset::default();
        for syntax in &self.parser {
            recurse(syntax, &mut HashMap::new(), &mut pos, &mut output_vars);
        }

        self.vars = Some(output_vars)
    }
}
