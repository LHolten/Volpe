use std::collections::HashMap;
use std::sync::Arc;
use volpe_parser::{lexeme_kind::LexemeKind, offset::Offset, packrat::Parser, syntax::RuleKind};

pub struct Variable {
    pub declaration: Offset,
    pub scope: u32,
    pub parameter: bool,
}

impl Variable {
    fn new(declaration: Offset, scope: u32, parameter: bool) -> Variable {
        Variable {
            declaration,
            scope,
            parameter,
        }
    }
}

pub fn pass(parser: &Parser) -> HashMap<Offset, Arc<Variable>> {
    let mut output_variables = HashMap::new();

    // Track which identifiers are in scope.
    let mut scoped_variables: HashMap<&str, Vec<Arc<Variable>>> = HashMap::new();

    // Traverse parse tree lexeme by lexeme.
    let mut pos = Offset::default();
    let mut next_lexemes = vec![(parser.0.as_ref(), 0)];
    while let Some((lexeme, mut scope)) = next_lexemes.pop() {
        for (i, rule) in lexeme.rules.iter().enumerate() {
            if rule.length == Offset::default() {
                continue;
            }
            if let Some(next) = &rule.next {
                next_lexemes.push((next, scope));
                // Entering a function increments scope.
                // It is returned back to the previous value when we pop
                // the next lexeme that this rule was pointing to (i.e. after this rule)
                if matches!(RuleKind::from(i), RuleKind::Func) {
                    scope += 1;
                }
            }
        }

        // Track scope.
        match &lexeme.kind {
            LexemeKind::LBrace => scope += 1,
            LexemeKind::RBrace => scope -= 1,
            _ => {}
        }

        match &lexeme.kind {
            LexemeKind::RBrace => {
                for (_, variables) in scoped_variables.iter_mut() {
                    if let Some(var) = variables.last() {
                        if var.scope >= scope {
                            variables.pop();
                        }
                    }
                }
            }
            LexemeKind::Ident => {
                let name = &lexeme.string[..lexeme.token_length.char as usize];

                if matches!(
                    &lexeme.next, Some(next) if matches!(next.kind, LexemeKind::Func | LexemeKind::Assign)
                ) {
                    // A new variable has been assigned.
                    let variables = scoped_variables.entry(name).or_insert(Vec::new());
                    let var = Arc::new(Variable::new(
                        pos,
                        scope,
                        matches!(&lexeme.next, Some(next) if matches!(next.kind, LexemeKind::Func)),
                    ));
                    output_variables.insert(pos, Arc::clone(&var));
                    variables.push(var);
                } else {
                    // Connect the use of a variable to its declaration.
                    if let Some(variables) = scoped_variables.get(name) {
                        if let Some(var) = variables.last() {
                            if var.scope <= scope {
                                output_variables.insert(pos, Arc::clone(&var));
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // Follow lexeme chains.
        if let Some(next) = &lexeme.next {
            next_lexemes.push((next, scope));
        }

        // Track current position in text.
        pos += lexeme.length;
    }

    output_variables
}
