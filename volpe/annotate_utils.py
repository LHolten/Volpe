from itertools import count
from typing import Dict

from lark import Token
from llvmlite import ir

from tree import TypeTree
from volpe_types import VolpeObject, int1, int32, pint8, VolpeClosure


def logic(self, tree: TypeTree):
    ret = self.visit_children(tree)
    assert ret[0] == int1 and ret[1] == int1, "logic operations only work for booleans"
    return int1


def unary_logic(self, tree: TypeTree):
    ret = self.visit_children(tree)[0]
    assert ret == int1, "unary logic operations only work for booleans"
    return int1


def math(self, tree: TypeTree):
    children = self.visit_children(tree)
    ret0 = children[0]
    ret1 = children[1]
    assert ret0 == ret1, "types need to match for math operations"
    if ret0 == int32:
        tree.data = tree.data + "_int"
    elif ret0 == self.flt:
        tree.data = tree.data + "_flt"
    else:
        raise AssertionError("math operations only work for integers and floats")
    return ret0


def unary_math(self, tree: TypeTree):
    ret = self.visit_children(tree)[0]
    if ret == int32:
        tree.data = tree.data + "_int"
    elif ret == self.flt:
        tree.data = tree.data + "_flt"
    else:
        raise AssertionError("unary math operations only work for integers and floats")
    return ret


def math_assign(self, tree: TypeTree):
    symbol = tree.children[0]
    symbol.data = "symbol"
    expression = tree.children[1]

    # Make the current node an assign.
    operation = tree.data.replace("_assign", "")
    tree.data = "assign"

    # Make a child node with the math operation.
    tree.children[1] = TypeTree(operation, [symbol, expression])

    return self.visit(tree)


def comp(self, tree: TypeTree):
    children = self.visit_children(tree)
    ret0 = children[0]
    ret1 = children[1]
    assert ret0 == ret1, "types need to match for comparisons"
    if ret0 == int32:
        tree.data = tree.data + "_int"
    elif ret0 == self.flt:
        tree.data = tree.data + "_flt"
    else:
        raise AssertionError("comparisons only work for integers and floats")
    return int1


def tuple_assign(scope: Dict, tree: TypeTree, value_type):
    if tree.data == "object":
        assert isinstance(value_type, VolpeObject), "can only destructure objects"
        assert len(tree.children) == len(value_type.type_dict.values())

        for i, child in enumerate(tree.children):
            tuple_assign(scope, child, value_type.type_dict[f"_{i}"])
    else:
        assert tree.data == "symbol"
        scope[tree.children[0].value] = value_type

    return scope


def func_ret(closure, arg_types):
    def ret(value_type):
        if closure.block.return_type is not None:
            assert closure.block.return_type == value_type, "different return types encountered in same block"
        else:
            closure.block.return_type = value_type
        closure.update(ir.FunctionType(value_type, [pint8, *arg_types]))

    return ret