use crate::lsp_utils::range;
use lsp_types::{Diagnostic, DiagnosticSeverity};
use volpe_parser_2::{error::SyntaxError, file::File, offset::Offset};
use volpe_compiler_2::compile;

pub struct Document {
    pub version: i32,
    pub file: File,
}

impl Document {
    pub fn new(params: &lsp_types::DidOpenTextDocumentParams) -> Document {
        let mut file = File::default();
        file.patch(
            Offset::default(),
            Offset::default(),
            params.text_document.text.to_string(),
        )
        .unwrap();
        Document {
            version: params.text_document.version,
            file,
        }
    }

    pub fn update(&mut self, params: &lsp_types::DidChangeTextDocumentParams) {
        self.version = params.text_document.version;

        for event in params.content_changes.iter() {
            if let Some(range) = event.range {
                // TODO fix utf-8 and utf-16 mismatch
                let start = Offset::new(range.start.line as usize, range.start.character as usize);
                let end = Offset::new(range.end.line as usize, range.end.character as usize);
                self.file
                    .patch(start, end - start, event.text.to_string())
                    .unwrap();
            }
        }
    }

    pub fn get_info(&self) -> String {
        format!("version: {}\nresult: {}\n{:#}", self.version, self.compile_and_run().unwrap_or_else(|| "...".to_string()), self.file.rule())
    }

    pub fn get_diagnostics(&self) -> Vec<Diagnostic> {
        self.file
            .rule()
            .iter_errs()
            .filter(SyntaxError::should_show)
            .map(|error| {
                let (start, end) = error.get_range();
                Diagnostic {
                    range: range(start, end),
                    severity: Some(DiagnosticSeverity::Error),
                    source: Some("Volpe Language Server".to_string()),
                    message: format!("{}", error),
                    ..Default::default()
                }
            })
            .collect()
    }

    // TEMP
    pub fn compile_and_run(&self) -> Option<String> {
        if self.file.rule().iter_errs().next().is_none() {
            let instance = compile(&self.file).ok()?;
            let main = instance.exports.get_function("main").ok()?;
            let result = main.call(&[]).ok()?;
            return Some(format!("{:?}", result[0]));
        }
        None
    }
}
