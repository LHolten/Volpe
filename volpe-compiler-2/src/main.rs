use std::collections::HashMap;

use volpe_parser_2::{ast::ASTBuilder, file::File, offset::Offset};
use wasm_encoder::{
    CodeSection, Export, ExportSection, Function, FunctionSection, Module, TypeSection, ValType,
};
use wasmer::{imports, Instance};

use crate::wasm::{Compiler, Signature};

mod wasm;

fn main() {
    let mut file = File::default();
    file.patch(
        Offset::default(),
        Offset::default(),
        "a:(a + 2) 3".to_string(),
    )
    .unwrap();
    println!("{}", file.rule());

    let ast = ASTBuilder::default().convert(&Box::new(file.rule().unwrap()));

    let mut compiler = Compiler {
        signatures: HashMap::new(),
        functions: vec![],
        func: Function::new(vec![]),
    };

    let index = compiler.compile_new(&Signature {
        expression: ast,
        arg_stack: vec![],
    });

    let mut exports = ExportSection::new();
    exports.export("test", Export::Function(index as u32));

    let mut types = TypeSection::new();
    let mut functions = FunctionSection::new();
    let mut codes = CodeSection::new();
    for (t, (signature, func)) in compiler.functions.iter().enumerate() {
        let strict_len = signature.expression.strict_len();
        types.function((0..strict_len).map(|_| ValType::I32), vec![ValType::I32]);
        functions.function(t as u32);
        codes.function(func);
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
    let instance = Instance::new(&module, &import_object).unwrap();

    let test = instance.exports.get_function("test").unwrap();
    let result = test.call(&[]).unwrap();
    assert_eq!(result[0], wasmer::Value::I32(5));
}
