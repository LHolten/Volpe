from typing import Dict

from llvmlite import ir

from builder_utils import Closure
from tree import TypeTree
from volpe_types import VolpeTuple


class Unannotated(Closure):
    def __init__(self, scope: Dict, arg_names, code):
        super().__init__(ir.FunctionType(ir.VoidType(), []))
        self.scope = scope
        self.arg_names = arg_names
        self.code = code
        self.checked = False

    def update(self, func: ir.FunctionType):
        super().__init__(func)


def tuple_assign(scope: Dict, tree: TypeTree, value_type):
    if tree.data == "shape":
        assert isinstance(value_type, VolpeTuple), "can only destructure tuples"
        assert len(tree.children) == len(value_type.elements)

        for i, child in enumerate(tree.children):
            tuple_assign(scope, child, value_type.elements[i])
    else:
        scope[tree.children[0].value] = value_type


