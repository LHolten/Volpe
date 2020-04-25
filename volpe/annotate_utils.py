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


def tuple_assign(self, scope: Dict, tree: TypeTree, value_type):
    if tree.data == "object":
        assert isinstance(value_type, VolpeObject), "can only destructure objects"

        def ret(value_type):
            assert False, "can't return from an object"

        c = count()
        for i, child in enumerate(tree.children):
            if child.data not in {"assign", "add_assign", "sub_assign", "div_assign", "mul_assign"}:
                tree.children[i] = TypeTree("assign", [child, TypeTree("symbol", [Token("", f"_{next(c)}")])])

        obj = self.__class__(tree, self.get_scope, value_type.type_dict, ret)
        scope[obj.local_scope.keys()] = obj.local_scope.values()
    else:
        scope[tree.children[0].value] = value_type


def func_ret(closure, arg_types):
    def ret(value_type):
        if closure.block.return_type is not None:
            assert closure.block.return_type == value_type, "different return types encountered in same block"
        else:
            closure.block.return_type = value_type
        closure.update(ir.FunctionType(value_type, [pint8, *arg_types]))

    return ret


def closure_call(self, closure: VolpeClosure, arg_types):
    assert len(closure.arg_names) == len(arg_types), "func call with wrong number of arguments"

    if closure.checked:  # we have already been here
        return closure.func.return_type
    closure.checked = True

    args = dict()
    for a, t in zip(closure.arg_names, arg_types):
        tuple_assign(self, args, a, t)
    args["@"] = closure

    if closure.block.data != "block":
        closure.block = TypeTree("block", [TypeTree("return_n", [closure.block])])

    def scope(name):
        if name in args.keys():
            return args[name]
        return closure.get_scope

    self.__class__(closure.block, closure.get_scope, args, func_ret(closure, arg_types))
