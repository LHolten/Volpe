from typing import Dict, Callable

from lark.visitors import Interpreter

from annotate_utils import tuple_assign, logic, unary_logic, math, unary_math, math_assign, comp, func_ret
from tree import TypeTree
from volpe_types import int1, int32, flt32, VolpeTuple, Closure


class AnnotateScope(Interpreter):
    def __init__(self, scope: Dict, tree: TypeTree, ret: Callable):
        self.scope = scope
        self.ret = ret

        if tree.data == "block":
            values = self.visit_children(tree)  # sets tree.ret
            assert all([v == int1 for v in values]), "some line does not evaluate to a bool"
        else:
            ret(self.visit(tree))

        assert tree.ret, "void methods should return true"

    def visit(self, tree: TypeTree):
        tree.ret = getattr(self, tree.data)(tree)
        return tree.ret

    def block(self, tree: TypeTree):
        def ret(value_type):
            if tree.ret is not None:
                assert tree.ret == value_type, "different return types encountered in same block"
            else:
                tree.ret = value_type

        AnnotateScope(self.scope.copy(), tree, ret)

        return tree.ret

    def func(self, tree: TypeTree):
        return Closure(self.scope.copy(), tree.children[:-1], tree.children[-1])

    def func_call(self, tree: TypeTree):
        closure = self.visit(tree.children[0])
        arg_types = [self.visit(child) for child in tree.children[1:]]

        assert isinstance(closure, Closure)
        assert len(closure.arg_names) == len(arg_types), "func call with wrong number of arguments"

        if closure.checked:  # we have already been here
            return closure.func.return_type
        closure.checked = True

        scope = closure.outside_scope
        for a, t in zip(closure.arg_names, arg_types):
            tuple_assign(scope, a, t)

        AnnotateScope(scope, closure.block, func_ret(closure, arg_types))

        return closure.func.return_type

    def this_func(self, tree: TypeTree):
        return self.closure

    def returnn(self, tree: TypeTree):
        self.ret(self.visit(tree.children[0]))
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
