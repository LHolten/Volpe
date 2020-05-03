import itertools
from os import path

from lark import Lark
from llvmlite import ir

from annotate import AnnotateScope, FastAnnotateScope
from annotate_utils import func_ret
from builder import LLVMScope, FastLLVMScope
from builder_utils import build_func
from compile import compile_and_run
from tree import TypeTree
from volpe_types import pint8, int32, VolpeObject, VolpeList, target_data, VolpeClosure, copy_func, free_func


def volpe_llvm(tree: TypeTree, verbose=False, show_time=False, fast=False):
    if verbose:
        print(tree.pretty())

    def scope(name):
        raise AssertionError("not in scope: " + name)

    func_type = VolpeClosure(scope, {}, [], tree)
    func_type.checked = True

    if fast:
        FastAnnotateScope(tree, func_type.get_scope, func_ret(func_type, []))
    else:
        AnnotateScope(tree, func_type.get_scope, func_ret(func_type, []))

    if verbose:
        print(tree.pretty())

    module = ir.Module("program")
    module.func_count = itertools.count()
    module.malloc = ir.Function(module, ir.FunctionType(pint8, [int32]), "malloc")
    module.free = ir.Function(module, ir.FunctionType(ir.VoidType(), [pint8]), "free")
    module.memcpy = module.declare_intrinsic('llvm.memcpy', [pint8, pint8, int32])

    func = ir.Function(module, func_type.func, "actual_main")
    c_func = ir.Function(module, copy_func, "actual_main.copy")
    f_func = ir.Function(module, free_func, "actual_main.free")
    closure = func_type([func, c_func, f_func, ir.Undefined])

    def scope(name):
        assert name == "@"
        return closure

    with build_func(func) as (b, args):
        if fast:
            FastLLVMScope(b, tree, scope, b.ret, None)
        else:
            LLVMScope(b, tree, scope, b.ret, None)

    with build_func(c_func) as (b, args):
        b.ret(pint8(ir.Undefined))

    with build_func(f_func) as (b, args):
        b.ret_void()

    return_type = func_type.func.return_type
    # return as pointer so they can be printed more easily
    if isinstance(return_type, (VolpeObject, VolpeList)):
        return_type = return_type.as_pointer()

    main_func = ir.Function(module, ir.FunctionType(return_type, []), "main")
    with build_func(main_func) as (b, _):
        b: ir.IRBuilder
        res = b.call(func, [pint8(ir.Undefined)])
        if isinstance(res.type, (VolpeObject, VolpeList)):
            ptr = b.bitcast(b.call(module.malloc, [int32(res.type.get_abi_size(target_data))]), return_type)
            b.store(res, ptr)
            res = ptr
        b.ret(res)

    if verbose:
        print(module)

    compile_and_run(str(module), tree.return_type, show_time=show_time)
    # return scope.visit(tree)


def run(file_path, verbose=False, show_time=False, fast=False):
    base_path = path.dirname(__file__)
    path_to_lark = path.abspath(path.join(base_path, "volpe.lark"))
    with open(path_to_lark) as lark_file:
        volpe_parser = Lark(lark_file, start='block', parser='earley', ambiguity='explicit', tree_class=TypeTree)
    with open(file_path) as vlp_file:
        parsed_tree = volpe_parser.parse(vlp_file.read())
    # print(parsed_tree.pretty())
    volpe_llvm(parsed_tree, verbose=verbose, show_time=show_time, fast=fast)
    # llvm_ir()
