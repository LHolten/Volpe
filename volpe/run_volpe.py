import itertools
from os import path

from lark import Lark
from llvmlite import ir
from unification import var, reify

from annotate import AnnotateScope
from builder import LLVMScope
from builder_utils import build_func
from compile import compile_and_run
from tree import TypeTree
from volpe_types import pint8, VolpeObject, VolpeList, target_data, VolpeClosure, int64, unwrap


def volpe_llvm(tree: TypeTree, verbose=False, show_time=False):
    if verbose:
        print(tree.pretty())

    closure = VolpeClosure(VolpeObject(dict()), var())
    arg_scope = {"@": closure}

    def scope(name):
        if name in arg_scope:
            return arg_scope[name]
        assert False, f"variable `{name}` not found"

    rules = AnnotateScope(tree, scope, dict(), closure.ret_type).rules

    def update(t: TypeTree):
        t.return_type = reify(t.return_type, rules)
        for child in t.children:
            if isinstance(child, TypeTree):
                update(child)

    update(tree)

    if verbose:
        print(tree.pretty())

    module = ir.Module("program")
    module.func_count = itertools.count()
    module.malloc = ir.Function(module, ir.FunctionType(pint8, [int64]), "malloc")
    module.free = ir.Function(module, ir.FunctionType(ir.VoidType(), [pint8]), "free")
    module.memcpy = module.declare_intrinsic('llvm.memcpy', [pint8, pint8, int64])

    return_type = unwrap(tree.return_type)
    # return as pointer so they can be printed more easily
    if isinstance(return_type, ir.LiteralStructType):
        return_type = return_type.as_pointer()

    main_func = ir.Function(module, ir.FunctionType(return_type, []), "main")
    with build_func(main_func) as (b, _):
        def ret(res):
            if isinstance(res.type, ir.LiteralStructType):
                ptr = b.bitcast(b.call(module.malloc, [int64(res.type.get_abi_size(target_data))]), return_type)
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
