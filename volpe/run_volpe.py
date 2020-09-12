import itertools
from os import path
import traceback
from typing import Dict

from lark import Lark
from lark.exceptions import UnexpectedEOF, UnexpectedCharacters
from llvmlite import ir

from annotate import AnnotateScope
from builder import LLVMScope
from builder_utils import build_func
from compile import compile_and_run
from tree import TypeTree, VolpeError
from volpe_types import unwrap, unknown


base_path = path.dirname(__file__)
path_to_lark = path.abspath(path.join(base_path, "volpe.lark"))
with open(path_to_lark) as lark_file:
    volpe_parser = Lark(
        lark_file,
        start="block",
        parser="earley",
        ambiguity="explicit",
        tree_class=TypeTree,
        propagate_positions=True,
    )


def volpe_llvm(tree: TypeTree, verbose=False, show_time=False, more_verbose=False, console=False):
    if more_verbose:
        print(tree.pretty())

    arg_scope = {}

    def scope(name, local_tree: TypeTree):
        if name in arg_scope:
            return arg_scope[name]
        raise VolpeError(f"variable `{name}` not found", local_tree)

    AnnotateScope(tree, scope)

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

    compile_and_run(str(module), tree.return_type, more_verbose=more_verbose, show_time=show_time, console=console)


def parse_trees(file_path: str, imports: Dict):
    if file_path in imports:
        return

    with open(file_path) as vlp_file:
        try:
            tree = volpe_parser.parse(vlp_file.read())
            obj_tree = TypeTree("object", [], tree.meta)
            imports[file_path] = TypeTree("func", [obj_tree, tree], tree.meta)
        except UnexpectedCharacters as err:
            # Return cursor to start of file.
            vlp_file.seek(0)
            line = vlp_file.readlines()[err.line - 1]
            symbol = line[err.column - 1]
            # Print the line.
            error_message = f"unexpected symbol '{symbol}'"
            error_message += f"\n{err.line}| {line}"
            # Add the cursor.
            padding = " " * (len(str(err.line)) + err.column)
            error_message += f"\n{padding} ^"
            raise VolpeError(error_message)

    for subtree in imports[file_path].iter_subtrees():
        subtree.meta.file_path = file_path

        if subtree.data == "import_":
            directory = path.dirname(file_path)
            import_path = path.join(directory, *[child.value for child in subtree.children]) + ".vlp"
            parse_trees(import_path, imports)
            subtree.data = "func_call"
            subtree.children = [imports[import_path], obj_tree]

    return tree


def run(file_path, verbose=False, show_time=False, console=False):
    try:
        imports = dict()
        tree = parse_trees(file_path, imports)
        volpe_llvm(tree, verbose, show_time, verbose, console)
    except UnexpectedEOF:
        print("unexpected end-of-input")
    except VolpeError as err:
        if verbose:
            traceback.print_exc()
        else:
            print(err)
