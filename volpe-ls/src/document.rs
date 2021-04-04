use volpe_parser::{offset::Offset, packrat::Parser};

pub struct Document {
    pub version: i32,
    pub parser: Parser,
    pub vars: Vec<Variable>,
}

impl Document {
    pub fn new(params: &lsp_types::DidOpenTextDocumentParams) -> Document {
        let mut parser = Parser::default();
        parser.parse(
            &params.text_document.text,
            Offset::default(),
            Offset::default(),
        );
        Document {
            version: params.text_document.version,
            vars: get_vars(&parser),
            parser,
        }
    }

    pub fn update(&mut self, params: &lsp_types::DidChangeTextDocumentParams) {
        self.version = params.text_document.version;

        for event in params.content_changes.iter() {
            if let Some(range) = event.range {
                // TODO fix utf-8 and utf-16 mismatch
                let start = Offset::new(range.start.line, range.start.character);
                let end = Offset::new(range.end.line, range.end.character);
                self.parser.parse(&event.text, start, end - start);
            }
        }

        self.vars = get_vars(&self.parser);
    }

    pub fn get_info(&self) -> String {
        format!("version: {}\n{}", self.version, self.parser)
    }
}

pub struct Variable {
    pub name: String,
    pub declaration: Offset,
    pub scope: u32,
    pub locations: Vec<Offset>,
    pub parameter: bool,
}

impl Variable {
    fn new(name: String, declaration: Offset, scope: u32, parameter: bool) -> Variable {
        Variable {
            name,
            declaration,
            scope,
            locations: Vec::new(),
            parameter,
        }
    }
}

use std::collections::{HashMap, HashSet};
use volpe_parser::{lexeme_kind::LexemeKind, syntax::RuleKind};

// Variable pass
fn get_vars(parser: &Parser) -> Vec<Variable> {
    let mut variables = Vec::new();
    let mut scoped_identifiers: HashMap<&str, HashSet<u32>> = HashMap::new();

    let mut pos = Offset::default();
    let mut next_lexemes = vec![(parser.0.as_ref(), 0)];
    while let Some((lexeme, mut scope)) = next_lexemes.pop() {
        for (i, rule) in lexeme.rules.iter().enumerate() {
            if rule.length == Offset::default() {
                continue;
            }
            if let Some(next) = &rule.next {
                next_lexemes.push((next, scope));
                if matches!(RuleKind::from(i), RuleKind::Func) {
                    scope += 1;
                }
            }
        }

        match &lexeme.kind {
            LexemeKind::LBrace => scope += 1,
            LexemeKind::RBrace => {
                for (_, scopes) in scoped_identifiers.iter_mut() {
                    scopes.remove(&scope);
                }
                scope -= 1;
            }
            LexemeKind::Ident => {
                let name = &lexeme.string[..lexeme.token_length.char as usize];
                if matches!(
                    &lexeme.next, Some(next) if matches!(next.kind, LexemeKind::Func | LexemeKind::Assign)
                ) {
                    let scopes = scoped_identifiers.entry(name).or_insert(HashSet::new());
                    scopes.insert(scope);
                    variables.push(Variable::new(
                        name.to_string(),
                        pos,
                        scope,
                        matches!(&lexeme.next, Some(next) if matches!(next.kind, LexemeKind::Func)),
                    ));
                } else {
                    // Variables are ordered, so going from back means we get the latest definition.
                    // We need to check scope in case of shadowing.
                    for var in variables.iter_mut().rev() {
                        if var.name == name && var.scope <= scope {
                            var.locations.push(pos);
                            break;
                        }
                    }
                }
            }
            _ => {}
        }

        if let Some(next) = &lexeme.next {
            next_lexemes.push((next, scope));
        }

        pos += lexeme.length;
    }
    variables
}
