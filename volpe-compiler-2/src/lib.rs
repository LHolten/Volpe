use volpe_parser_2::{ast::ASTBuilder, file::File, stack_list::StackList};
use wasm_encoder::{
    CodeSection, Export, ExportSection, FunctionSection, Module, TypeSection, ValType,
};
use wasmer::{imports, Instance};

use crate::wasm::{Compiler, Signature};

mod wasm;

// TEMPORARY
pub fn compile(file: &File) -> Result<Instance, Box<dyn std::error::Error>> {
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
        codes.function(entry.function.as_ref().ok_or("")?);
    }

    let mut module = Module::new();
    module.section(&types);
    module.section(&functions);
    module.section(&exports);
    module.section(&codes);

    // Extract the encoded Wasm bytes for this module.
    let wasm_bytes = module.finish();

    let module = wasmer::Module::from_binary(&wasmer::Store::default(), &wasm_bytes)?;
    // The module doesn't import anything, so we create an empty import object.
    let import_object = imports! {};
    let instance = Instance::new(&module, &import_object)?;
    Ok(instance)
}

#[cfg(test)]
mod tests {
    use super::compile;
    use volpe_parser_2::{file::File, offset::Offset};

    #[test]
    fn conditional() {
        let mut file = File::default();
        file.patch(
            Offset::default(),
            Offset::default(),
            "(Test: 5, Beta: 10, ()) Beta".to_string(),
        )
        .unwrap();

        let instance = compile(&file).unwrap();
        let main = instance.exports.get_function("main").unwrap();
        let result = main.call(&[]).unwrap();
        assert_eq!(result[0], wasmer::Value::I32(10));
    }

    #[test]
    fn recursion() {
        let mut file = File::default();
        file.patch(
            Offset::default(),
            Offset::default(),
            "f.(x.(f (x x)) x.(f (x x))) rec.unique.((A: (rec B), B: 1, ()) unique) A".to_string(),
        )
        .unwrap();

        let instance = compile(&file).unwrap();
        let main = instance.exports.get_function("main").unwrap();
        let result = main.call(&[]).unwrap();
        assert_eq!(result[0], wasmer::Value::I32(1));
    }

    #[test]
    fn assign() {
        let mut file = File::default();
        file.patch(
            Offset::default(),
            Offset::default(),
            "
            with = assign.inner.outer.(
                inner (assign, outer)
            );
            
            with Alpha:{10} (
                with Beta:{5} {
                    Alpha x.x
                }
            ) ()
            "
            .to_string(),
        )
        .unwrap();

        let instance = compile(&file).unwrap();
        let main = instance.exports.get_function("main").unwrap();
        let result = main.call(&[]).unwrap();
        assert_eq!(result[0], wasmer::Value::I32(10));
    }

    #[test]
    fn test_wasm() {
        let mut file = File::default();
        file.patch(
            Offset::default(),
            Offset::default(),
            "
            #2{(i32.add)} 1 2
            "
            .to_string(),
        )
        .unwrap();

        let instance = compile(&file).unwrap();
        let main = instance.exports.get_function("main").unwrap();
        let result = main.call(&[]).unwrap();
        assert_eq!(result[0], wasmer::Value::I32(3));
    }

    #[test]
    fn test_wasm_2() {
        let mut file = File::default();
        file.patch(
            Offset::default(),
            Offset::default(),
            "
            strict = #0{(return_call 0)};
            strict 10
            "
            .to_string(),
        )
        .unwrap();

        let instance = compile(&file).unwrap();
        let main = instance.exports.get_function("main").unwrap();
        let result = main.call(&[]).unwrap();
        assert_eq!(result[0], wasmer::Value::I32(10));
    }

    #[test]
    fn test_wasm_3() {
        let mut file = File::default();
        file.patch(
            Offset::default(),
            Offset::default(),
            "
            bool = #1{if (result i32) (return_call 0) else (return_call 1) end} 1 0;
            bool 10
            "
            .to_string(),
        )
        .unwrap();

        let instance = compile(&file).unwrap();
        let main = instance.exports.get_function("main").unwrap();
        let result = main.call(&[]).unwrap();
        assert_eq!(result[0], wasmer::Value::I32(1));
    }
}
