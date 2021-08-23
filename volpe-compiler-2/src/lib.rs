use volpe_parser_2::{ast::ASTBuilder, file::File, stack_list::StackList};
use wasm_encoder::{
    CodeSection, Export, ExportSection, FunctionSection, Module, TypeSection, ValType,
};
use wasmer::{imports, Instance};

use crate::wasm::{Compiler, Signature};

mod wasm;

// TEMPORARY
pub fn compile(file: File) -> Instance {
    let ast = ASTBuilder::default().convert(StackList::default(), &Box::new(file.rule().unwrap()));

    let mut compiler = Compiler(vec![]);

    let index = compiler.compile_new(&Signature {
        expression: ast,
        arg_stack: vec![],
    });

    let mut exports = ExportSection::new();
    exports.export("main", Export::Function(index as u32));

    let mut types = TypeSection::new();
    let mut functions = FunctionSection::new();
    let mut codes = CodeSection::new();
    for (t, entry) in compiler.0.iter().enumerate() {
        let strict_len = entry.signature.expression.strict_len();
        types.function((0..strict_len).map(|_| ValType::I32), vec![ValType::I32]);
        functions.function(t as u32);
        codes.function(entry.function.as_ref().unwrap());
    }

    let mut module = Module::new();
    module.section(&types);
    module.section(&functions);
    module.section(&exports);
    module.section(&codes);

    // Extract the encoded Wasm bytes for this module.
    let wasm_bytes = module.finish();

    let module = wasmer::Module::from_binary(&wasmer::Store::default(), &wasm_bytes).unwrap();
    // The module doesn't import anything, so we create an empty import object.
    let import_object = imports! {};
    Instance::new(&module, &import_object).unwrap()
}

#[cfg(test)]
mod tests {
    use volpe_parser_2::{file::File, offset::Offset};
    use super::compile;

    #[test]
    fn strict_func() {
        let mut file = File::default();
        file.patch(
            Offset::default(),
            Offset::default(),
            "a:(a + 2) 3".to_string(),
        )
        .unwrap();

        let instance = compile(file);
        let main = instance.exports.get_function("main").unwrap();
        let result = main.call(&[]).unwrap();
        assert_eq!(result[0], wasmer::Value::I32(5));
    }
}