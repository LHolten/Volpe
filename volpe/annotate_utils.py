from typing import Dict

from tree import TypeTree
from volpe_types import VolpeTuple


def tuple_assign(scope: Dict, tree: TypeTree, value_type):
    if tree.data == "shape":
        assert isinstance(value_type, VolpeTuple), "can only destructure tuples"
        assert len(tree.children) == len(value_type.elements)

        for i, child in enumerate(tree.children):
            tuple_assign(scope, child, value_type.elements[i])
    else:
        scope[tree.children[0].value] = value_type


