import itertools

from os import path

from lark import Lark
from llvmlite import ir

from annotate import AnnotateScope, Unannotated
from compile import compile_and_run
from builder import LLVMScope, build_function
from volpe_types import pint8, int32, make_int
from tree import TypeTree


def volpe_llvm(tree: TypeTree, verbose=False):
    if verbose:
        print(tree.pretty())

    closure = Unannotated({}, [], tree)
    closure.update(ir.FunctionType(ir.VoidType(), [pint8]))
    AnnotateScope({}, tree, closure, True)

    if verbose:
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

    if verbose:
        print(module)

    compile_and_run(str(module), tree.ret)
    # return scope.visit(tree)


def run(file_path, verbose=False):
    base_path = path.dirname(__file__)
    path_to_lark = path.abspath(path.join(base_path, "volpe.lark"))
    with open(path_to_lark) as lark_file:
        volpe_parser = Lark(lark_file, start='code', parser='lalr', tree_class=TypeTree, debug=verbose)
    with open(file_path) as vlp_file:
        parsed_tree = volpe_parser.parse(vlp_file.read())
    # print(parsed_tree.pretty())
    volpe_llvm(parsed_tree, verbose)
    # llvm_ir()
