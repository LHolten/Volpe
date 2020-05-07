import itertools
from os import path

from lark import Lark
from llvmlite import ir

from annotate import AnnotateScope
from builder import LLVMScope
from builder_utils import build_func
from compile import compile_and_run
from tree import TypeTree
from volpe_types import pint8, int32, VolpeObject, VolpeList, target_data, VolpeClosure, VolpeBlock


def volpe_llvm(tree: TypeTree, verbose=False, show_time=False):
    if verbose:
        print(tree.pretty())

    def scope(name):
        raise AssertionError("not in scope: " + name)

    func_type = VolpeClosure(VolpeBlock(TypeTree("func", [TypeTree("object", []), tree]), lambda: scope))
    tree.return_type = func_type
    func_type.call(VolpeObject(dict()), AnnotateScope)

    if verbose:
        print(tree.pretty())

    module = ir.Module("program")
    module.func_count = itertools.count()
    module.malloc = ir.Function(module, ir.FunctionType(pint8, [int32]), "malloc")
    module.free = ir.Function(module, ir.FunctionType(ir.VoidType(), [pint8]), "free")
    module.memcpy = module.declare_intrinsic('llvm.memcpy', [pint8, pint8, int32])

    return_type = func_type.func.return_type
    # return as pointer so they can be printed more easily
    if isinstance(return_type, (VolpeObject, VolpeList)):
        return_type = return_type.as_pointer()

    main_func = ir.Function(module, ir.FunctionType(return_type, []), "main")
    with build_func(main_func) as (b, _):
        def ret(res):
            if isinstance(res.type, (VolpeObject, VolpeList)):
                ptr = b.bitcast(b.call(module.malloc, [int32(res.type.get_abi_size(target_data))]), return_type)
                b.store(res, ptr)
                res = ptr
            b.ret(res)

        LLVMScope(b, tree, scope, ret, None)

    if verbose:
        print(module)

    compile_and_run(str(module), tree.return_type, show_time=show_time)


def run(file_path, verbose=False, show_time=False):
    base_path = path.dirname(__file__)
    path_to_lark = path.abspath(path.join(base_path, "volpe.lark"))
    with open(path_to_lark) as lark_file:
        volpe_parser = Lark(lark_file, start='block', parser='earley', ambiguity='explicit', tree_class=TypeTree)
    with open(file_path) as vlp_file:
        parsed_tree = volpe_parser.parse(vlp_file.read())
    # print(parsed_tree.pretty())
    volpe_llvm(parsed_tree, verbose=verbose, show_time=show_time)
    # llvm_ir()
