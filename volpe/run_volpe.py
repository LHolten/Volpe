import itertools
from os import path
import traceback

from lark import Lark
from llvmlite import ir
from unification import var, reify, unify

from annotate import AnnotateScope
from builder import LLVMScope
from builder_utils import build_func, build_closure, free
from command_line import build_main, string_obj, string_type
from compile import compile_and_run
from tree import TypeTree, VolpeError
from volpe_types import pint8, VolpeObject, VolpeClosure, int64, unwrap, char, size, int32, VolpeArray, int1


def volpe_llvm(tree: TypeTree, verbose=False, show_time=False, more_verbose=False, console=False):
    if more_verbose:
        print(tree.pretty())

    closure = VolpeClosure(VolpeObject(dict()), var())
    printf = VolpeClosure(VolpeObject({"_0": VolpeArray(char)}), int64)
    arg_scope = {"$printf": printf}

    def scope(name, local_tree: TypeTree):
        if name in arg_scope:
            return arg_scope[name]
        raise VolpeError(f"variable `{name}` not found", local_tree)

    rules = AnnotateScope(tree, scope, dict(), closure.ret_type).rules

    if console:
        rules = unify(tree.return_type, VolpeClosure(string_obj, string_type), rules)
        assert rules is not False, "command line scripts should return a function that takes as list of strings " \
                                   "and returns a string"

    for t in tree.iter_subtrees():
        t.return_type = reify(t.return_type, rules)

    if verbose:
        print(tree.pretty())

    module = ir.Module("program")
    module.func_count = itertools.count()
    module.malloc = ir.Function(module, ir.FunctionType(pint8, [int64]), "malloc")
    module.free = ir.Function(module, ir.FunctionType(ir.VoidType(), [pint8]), "free")
    module.memcpy = module.declare_intrinsic('llvm.memcpy', [pint8, pint8, int64])
    module.printf = ir.Function(module, ir.FunctionType(int32, [char.as_pointer()]), "printf")

    with build_closure(module, printf, []) as (b, _, args, printf_func):
        string = b.extract_value(args[1], 0)
        res_len = b.extract_value(string, 1)
        new_string = b.call(module.malloc, [b.add(res_len, int64(1))])
        b.call(module.memcpy, [new_string, b.extract_value(string, 0), res_len, int1(False)])
        b.store(char(0), b.gep(new_string, [res_len]))
        free(b, string)

        code = b.call(module.printf, [new_string])
        b.call(module.free, [new_string])
        b.ret(b.zext(code, int64))

    return_type = unwrap(tree.return_type)
    # return as pointer so they can be printed more easily
    if isinstance(return_type, ir.LiteralStructType) and not console:
        return_type = return_type.as_pointer()

    run_func = ir.Function(module, ir.FunctionType(return_type, []), "run")
    with build_func(run_func) as (b, _):
        arg_scope = {"$printf": printf_func}

        def scope(name, mut):
            return arg_scope[name]

        def ret(res):
            if isinstance(res.type, ir.LiteralStructType) and not console:
                ptr = b.bitcast(b.call(module.malloc, [size(res.type)]), return_type)
                b.store(res, ptr)
                res = ptr
            b.ret(res)

        LLVMScope(b, tree, scope, ret, None)

    if console:
        build_main(module, run_func, printf_func)

    if more_verbose:
        print(module)

    compile_and_run(str(module), tree.return_type, show_time, console)


def get_parser(path_to_lark):
    with open(path_to_lark) as lark_file:
        return Lark(lark_file, start='block', parser='earley', ambiguity='explicit', tree_class=TypeTree,
                    propagate_positions=True)


def run(file_path, verbose=False, show_time=False, console=False):
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
            return
    # put file_path inside tree.meta so that VolpeError can print code blocks
    for tree in parsed_tree.iter_subtrees():
        tree.meta.file_path = file_path

    try:
        volpe_llvm(parsed_tree, verbose, show_time, verbose, console)
    except VolpeError as err:
        if verbose:
            traceback.print_exc()
        else:
            print(err)
