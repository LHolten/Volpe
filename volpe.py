import itertools

from lark import Lark
from llvmlite import ir

from annotate import AnnotateScope, Unannotated
from compile import compile_and_run
from llvm_builder import LLVMScope, build_function
from util import TypeTree, pint8, int32, make_int


def volpe_llvm(tree: TypeTree):
    print(tree.pretty())

    closure = Unannotated({}, [], tree)
    closure.update(ir.FunctionType(ir.VoidType(), [pint8]))
    AnnotateScope({}, tree, closure, True)

    print(tree.pretty())

    module = ir.Module("program")
    module.func_count = itertools.count()
    module.malloc = ir.Function(module, ir.FunctionType(pint8, [int32]), "malloc")
    module.free = ir.Function(module, ir.FunctionType(ir.VoidType(), [pint8]), "free")
    module.memcpy = module.declare_intrinsic('llvm.memcpy', [pint8, pint8, int32])

    func = ir.Function(module, closure.func, str(next(module.func_count)))
    build_function(func, [], [], [], tree)

    main_func = ir.Function(module, ir.FunctionType(closure.func.return_type, []), "main")
    builder = ir.IRBuilder(main_func.append_basic_block("start"))
    env = builder.call(module.malloc, [make_int(0)])
    builder.ret(builder.call(func, [env]))

    print(module)

    compile_and_run(str(module), tree.ret)
    # return scope.visit(tree)


def main():
    volpe_parser = Lark(open("volpe.lark"), start='code', parser='lalr', tree_class=TypeTree, debug=True)
    parsed_tree = volpe_parser.parse(open("test.vlp").read())
    # print(parsed_tree.pretty())
    volpe_llvm(parsed_tree)
    # llvm_ir()


if __name__ == '__main__':
    main()
