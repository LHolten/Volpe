import itertools
from os import path
import traceback

from lark import Lark
from lark.exceptions import UnexpectedEOF, UnexpectedCharacters
from llvmlite import ir
from unification import reify

from annotate import AnnotateScope
from builder import LLVMScope
from builder_utils import build_func
from compile import compile_and_run
from tree import TypeTree, VolpeError
from volpe_types import unwrap, unknown


def volpe_llvm(tree: TypeTree, verbose=False, show_time=False, more_verbose=False, console=False):
    if more_verbose:
        print(tree.pretty())

    arg_scope = {}

    def scope(name, local_tree: TypeTree):
        if name in arg_scope:
            return arg_scope[name]
        raise VolpeError(f"variable `{name}` not found", local_tree)

    rules = AnnotateScope(tree, scope, dict()).rules

    for t in tree.iter_subtrees():
        t.return_type = reify(t.return_type, rules)

    if verbose:
        print(tree.pretty())

    module = ir.Module("program")
    module.func_count = itertools.count()

    run_func = ir.Function(module, ir.FunctionType(unknown, [unwrap(tree.return_type).as_pointer()]), "run")
    with build_func(run_func) as (b, args):
        arg_scope = {}

        def scope(name):
            return arg_scope[name]

        def ret(value):
            b.store(value, args[0])
            b.ret_void()

        LLVMScope(b, tree, scope, ret, None)

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

        except UnexpectedEOF:
            print("unexpected end-of-input")
            return

        except UnexpectedCharacters as err:
            # Return cursor to start of file.
            vlp_file.seek(0)
            line = vlp_file.readlines()[err.line - 1]
            symbol = line[err.column - 1]
            # Print the line.
            error_message = f"unexpected symbol '{symbol}'"
            error_message += f"\n{err.line}| {line}"
            # Add the cursor.
            padding = " " * (len(str(err.line)) + len(line) - 1)
            error_message += f"\n{padding}  ^"
            print(error_message)
            return

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
