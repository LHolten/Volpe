from lark import Lark
from llvmlite import ir

from annotate import AnnotateScope
from compile import compile_and_run
from llvm_builder import LLVMScope
from util import TypeTree


def volpe_llvm(tree: TypeTree):
    # print(tree.pretty())

    AnnotateScope({}, {}, tree)

    print(tree.pretty())

    module = ir.Module("program")
    module.func_count = 0
    func_type = ir.FunctionType(tree.ret, ())
    func = ir.Function(module, func_type, "main")
    block = func.append_basic_block("entry")
    builder = ir.IRBuilder(block)
    LLVMScope(builder, {}, {}, tree)

    print(module)

    compile_and_run(str(module), func_type.return_type)
    # return scope.visit(tree)


def main():
    volpe_parser = Lark(open("volpe.lark"), start='code', parser='lalr', tree_class=TypeTree)
    parsed_tree = volpe_parser.parse(open("test.vlp").read())
    # print(parsed_tree.pretty())
    volpe_llvm(parsed_tree)
    # llvm_ir()


if __name__ == '__main__':
    main()
