from typing import Dict

from lark.visitors import Interpreter
from llvmlite import ir

from annotate_utils import tuple_assign, Unannotated
from volpe_types import int1, int32, pint8, flt32, VolpeTuple
from tree import TypeTree


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
    elif ret0 == flt32:
        tree.data = tree.data + "_flt"
    else:
        raise AssertionError("math operations only work for integers and floats")
    return ret0


def unary_math(self, tree: TypeTree):
    ret = self.visit_children(tree)[0]
    if ret == int32:
        tree.data = tree.data + "_int"
    elif ret == flt32:
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
    elif ret0 == flt32:
        tree.data = tree.data + "_flt"
    else:
        raise AssertionError("comparisons only work for integers and floats")
    return int1


class AnnotateScope(Interpreter):
    def __init__(self, scope: Dict, tree: TypeTree, closure: Unannotated, is_func):
        self.scope = scope
        self.tree = tree
        self.closure = closure
        self.is_func = is_func

        if tree.data == "code":
            values = self.visit_children(tree)  # sets self.ret
            assert all([v == int1 for v in values]), "some line does not evaluate to a bool"
        else:
            self.update_return(self.visit(tree))

        assert self.tree.ret, "void methods should return true"

    def update_return(self, ret):
        if self.tree.ret is not None:
            assert self.tree.ret == ret, "different return types encountered in same block"
        self.tree.ret = ret
        if self.is_func:
            func = ir.FunctionType(ret, self.closure.func.args)
            self.closure.update(func)

    def visit(self, tree: TypeTree):
        tree.ret = getattr(self, tree.data)(tree)
        return tree.ret

    def code(self, tree: TypeTree):
        AnnotateScope(self.scope.copy(), tree, self.closure, False)
        return tree.ret

    def func(self, tree: TypeTree):
        new_scope = self.scope.copy()

        return Unannotated(new_scope, tree.children[:-1], tree.children[-1])

    def func_call(self, tree: TypeTree) -> ir.Type:
        closure = self.visit(tree.children[0])
        arg_types = [self.visit(child) for child in tree.children[1:]]

        assert isinstance(closure, Unannotated)
        assert len(closure.arg_names) == len(arg_types), "func call with wrong number of arguments"

        if closure.checked:  # we have already been here
            return closure.func.return_type
        closure.checked = True

        scope = closure.scope
        for a, t in zip(closure.arg_names, arg_types):
            tuple_assign(scope, a, t)

        closure.update(ir.FunctionType(ir.VoidType(), [pint8, *arg_types]))

        AnnotateScope(scope, closure.code, closure, True)

        return closure.func.return_type

    def this_func(self, tree: TypeTree):
        return self.closure

    def returnn(self, tree: TypeTree):
        self.update_return(self.visit(tree.children[0]))
        return int1

    def symbol(self, tree: TypeTree):
        return self.scope[tree.children[0].value]

    def assign(self, tree: TypeTree):
        tuple_assign(self.scope, tree.children[0], self.visit(tree.children[1]))
        return int1

    def integer(self, tree: TypeTree):
        return int32

    def floating(self, tree: TypeTree):
        return flt32

    def convert(self, tree: TypeTree):
        ret = self.visit_children(tree)[0]
        if ret == int32:
            tree.data = tree.data + "_int"
            return flt32
        elif ret == flt32:
            tree.data = tree.data + "_flt"
            return int32
        else:
            raise AssertionError("convertion only work for integers and floats")

    def collect_tuple(self, tree: TypeTree):
        return VolpeTuple(self.visit_children(tree))

    # Boolean logic
    implication = logic
    logic_and = logic
    logic_or = logic
    logic_not = unary_logic

    # Mathematics
    add = math
    mod = math
    mul = math
    sub = math
    div = math
    # power = math
    negate = unary_math
    add_assign = math_assign
    sub_assign = math_assign
    mul_assign = math_assign
    div_assign = math_assign
    mod_assign = math_assign

    # Comparison
    equals = comp
    not_equals = comp
    greater = comp
    less = comp
    greater_equals = comp
    less_equals = comp

    def __default__(self, tree: TypeTree):
        raise NotImplementedError("annotate", tree.data)
