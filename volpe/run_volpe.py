import itertools
from os import path
import traceback

from lark import Lark
from llvmlite import ir
from unification import var, reify

from annotate import AnnotateScope
from builder import LLVMScope
from builder_utils import build_func
from compile import compile_and_run
from tree import TypeTree, VolpeError
from volpe_types import pint8, VolpeObject, VolpeList, target_data, VolpeClosure, int64, unwrap, check


def volpe_llvm(tree: TypeTree, verbose=False, show_time=False, more_verbose=False):
    if more_verbose:
        print(tree.pretty())

    closure = VolpeClosure(VolpeObject(dict()), var())
    arg_scope = {"@": closure}

    def scope(name):
        if name in arg_scope:
            return arg_scope[name]
        raise VolpeError(f"variable `{name}` not found")

    try:
        rules = AnnotateScope(tree, scope, dict(), closure.ret_type).rules
    except VolpeError as err:
        if more_verbose:
            traceback.print_exc()
        else:
            print(err)
        exit()

    failed = [False]

    def update(t: TypeTree):
        t.return_type = reify(t.return_type, rules)
        if not check(t.return_type):
            failed[0] = True
        for child in t.children:
            if isinstance(child, TypeTree):
                update(child)

    update(tree)

    if verbose:
        print(tree.pretty())

    assert not failed[0], "Some value has not been typed, run with verbose to see which"

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

        try:
            LLVMScope(b, tree, scope, ret, None)
        except VolpeError as err:
            if more_verbose:
                traceback.print_exc()
            else:
                print(err)
            exit()

    if more_verbose:
        print(module)

    compile_and_run(str(module), tree.return_type, show_time=show_time)


def get_parser(path_to_lark):
    with open(path_to_lark) as lark_file:
        return Lark(lark_file, start='block', parser='earley', ambiguity='explicit', tree_class=TypeTree, propagate_positions=True)


def run(file_path, verbose=False, show_time=False):
    base_path = path.dirname(__file__)
    path_to_lark = path.abspath(path.join(base_path, "volpe.lark"))
    volpe_parser = get_parser(path_to_lark)
    with open(file_path) as vlp_file:
        try:
            parsed_tree = volpe_parser.parse(vlp_file.read())
        except Exception as err:
            if verbose:
                traceback.print_exc()
            else:
                print(err)
            exit()
    # put file_path inside tree.meta so that VolpeError can print code blocks
    for tree in parsed_tree.iter_subtrees():
        tree.meta.file_path = file_path
    volpe_llvm(parsed_tree, verbose=verbose, show_time=show_time, more_verbose=verbose)
