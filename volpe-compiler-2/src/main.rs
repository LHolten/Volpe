use volpe_parser_2::{ast::ASTBuilder, file::File, offset::Offset};
use wasm_encoder::{
    CodeSection, Export, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};
use wasmer::{imports, Instance};

use crate::wasm::{Closure, Compiler};

mod wasm;

fn main() {
    let mut file = File::default();
    file.patch(Offset::default(), Offset::default(), "1 + 1".to_string())
        .unwrap();
    let ast = ASTBuilder::default().convert(&Box::new(file.rule().unwrap()));
    let closure = Closure {
        val: ast.into(),
        env: vec![],
    };

    let mut module = Module::new();

    // Encode the type section.
    let mut types = TypeSection::new();
    let params = vec![];
    let results = vec![ValType::I32];
    types.function(params, results);
    module.section(&types);

    // Encode the function section.
    let mut functions = FunctionSection::new();
    let type_index = 0;
    functions.function(type_index);
    module.section(&functions);

    // Encode the export section.
    let mut exports = ExportSection::new();
    exports.export("test", Export::Function(0));
    module.section(&exports);

    // Encode the code section.
    let mut compiler = Compiler {
        stack: vec![],
        func: Function::new(vec![]),
    };
    assert!(compiler.build(closure).is_num());
    compiler.func.instruction(Instruction::End);

    let mut codes = CodeSection::new();
    codes.function(&compiler.func);
    module.section(&codes);

    // Extract the encoded Wasm bytes for this module.
    let wasm_bytes = module.finish();

    let module = wasmer::Module::from_binary(&wasmer::Store::default(), &wasm_bytes).unwrap();
    // The module doesn't import anything, so we create an empty import object.
    let import_object = imports! {};
    let instance = Instance::new(&module, &import_object).unwrap();

    let test = instance.exports.get_function("test").unwrap();
    let result = test.call(&[]).unwrap();
    assert_eq!(result[0], wasmer::Value::I32(2));
}
