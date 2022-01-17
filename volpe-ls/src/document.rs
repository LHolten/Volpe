use crate::lsp_utils::range;
use lsp_types::{Diagnostic, DiagnosticSeverity};
use volpe_parser_2::{eval::Evaluator, file::File, offset::Offset};

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
        format!(
            "version: {}\n\nresult: {:?}\n\nast:{:#?}",
            self.version,
            self.compile_and_run(),
            self.file.rule().collect()
        )
    }

    pub fn get_diagnostics(&self) -> Vec<Diagnostic> {
        if let Err(errs) = self.file.rule().collect() {
            errs.into_iter()
                .map(|error| Diagnostic {
                    range: range(error.get_range()),
                    severity: Some(DiagnosticSeverity::Error),
                    source: Some("Volpe Language Server".to_string()),
                    message: format!("{}", error),
                    ..Default::default()
                })
                .collect()
        } else {
            Vec::with_capacity(0)
        }
    }

    // TEMP
    pub fn compile_and_run(&self) -> Result<String, String> {
        if let Ok(syntax) = self.file.rule().collect() {
            return Evaluator::eval(syntax.convert(vec![]));
        }
        Err("...".to_string())
    }
}
