import itertools

from os import path

from lark import Lark
from llvmlite import ir

from annotate import AnnotateScope
from annotate_utils import Unannotated
from builder import LLVMScope
from builder_utils import build_func, Closure
from compile import compile_and_run
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

    func = ir.Function(module, closure.func, "actual_main")

    with build_func(func) as (b, args):
        closure_value = ir.Constant(Closure(func.type), [func, ir.Undefined, ir.Undefined, ir.Undefined])

        LLVMScope(b, {}, tree, b.ret, set(), closure_value)

    main_func = ir.Function(module, ir.FunctionType(closure.func.return_type, []), "main")
    with build_func(main_func) as (b, _):
        res = b.call(func, [ir.Constant(pint8, ir.Undefined)])
        b.ret(res)

    if verbose:
        print(module)

    compile_and_run(str(module), tree.ret)
    # return scope.visit(tree)


def run(file_path, verbose=False):
    base_path = path.dirname(__file__)
    path_to_lark = path.abspath(path.join(base_path, "volpe.lark"))
    with open(path_to_lark) as lark_file:
        volpe_parser = Lark(lark_file, start='code', parser='earley', tree_class=TypeTree)
    with open(file_path) as vlp_file:
        parsed_tree = volpe_parser.parse(vlp_file.read())
    # print(parsed_tree.pretty())
    volpe_llvm(parsed_tree, verbose)
    # llvm_ir()
