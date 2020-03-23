from typing import Callable

from lark.visitors import Interpreter
from llvmlite import ir

from annotate_utils import tuple_assign, logic, unary_logic, math, unary_math, math_assign, comp, func_ret
from tree import TypeTree
from volpe_types import int1, int32, flt32, flt64, VolpeTuple, Closure, VolpeList, pint8

class AnnotateScope(Interpreter):
    flt = flt64

    def __init__(self, tree: TypeTree, scope: Callable, local_scope: dict, ret: Callable):
        self.scope = scope
        self.local_scope = local_scope
        self.ret = ret

        if tree.data == "block":
            values = self.visit_children(tree)  # sets tree.return_type
            assert all([v == int1 for v in values]), "some line does not evaluate to a bool"
        else:
            ret(self.visit(tree))

        assert tree.return_type, "void methods should return true"

    def get_scope(self, name):
        if name in self.local_scope:
            return self.local_scope[name]
        else:
            return self.scope(name)

    def visit(self, tree: TypeTree):
        tree.return_type = getattr(self, tree.data)(tree)
        return tree.return_type

    def block(self, tree: TypeTree):
        def ret(value_type):
            if tree.return_type is not None:
                assert tree.return_type == value_type, "different return types encountered in same block"
            else:
                tree.return_type = value_type

        # Make a new AnnotateScope or FastAnnotateScope.
        self.__class__(tree, self.get_scope, dict(), ret)

        return tree.return_type

    def func(self, tree: TypeTree):
        return Closure(self.scope, self.local_scope, tree.children[:-1], tree.children[-1])

    def func_call(self, tree: TypeTree):
        closure = self.visit(tree.children[0])
        arg_types = [self.visit(child) for child in tree.children[1:]]

        assert isinstance(closure, Closure)
        assert len(closure.arg_names) == len(arg_types), "func call with wrong number of arguments"

        if closure.checked:  # we have already been here
            return closure.func.return_type
        closure.checked = True

        args = dict()
        for a, t in zip(closure.arg_names, arg_types):
            tuple_assign(args, a, t)
        args["@"] = closure

        # Make a new AnnotateScope or FastAnnotateScope.
        self.__class__(closure.block, closure.get_scope, args, func_ret(closure, arg_types))

        return closure.func.return_type

    def this_func(self, tree: TypeTree):
        return self.get_scope("@")

    def returnn(self, tree: TypeTree):
        self.ret(self.visit(tree.children[0]))
        return int1

    def symbol(self, tree: TypeTree):
        return self.get_scope(tree.children[0].value)

    def assign(self, tree: TypeTree):
        tuple_assign(self.local_scope, tree.children[0], self.visit(tree.children[1]))
        return int1

    def integer(self, tree: TypeTree):
        return int32

    def number_list(self, tree: TypeTree):
        ret = self.visit_children(tree)
        assert ret[0] == int32
        assert ret[1] == int32
        closure = Closure(self.scope, self.local_scope, None, None)
        closure.update(ir.FunctionType(int32, [pint8, int32]))
        return VolpeList(closure)

    def list_index(self, tree: TypeTree):
        ret = self.visit_children(tree)
        assert isinstance(ret[0], VolpeList)
        assert ret[1] == int32
        return ret[0].closure.func.return_type

    def list_size(self, tree: TypeTree):
        ret = self.visit_children(tree)[0]
        assert isinstance(ret, VolpeList)
        return int32

    def convert_int(self, tree: TypeTree):
        assert self.visit(tree.children[0]) == int32
        return self.flt

    def convert_flt(self, tree: TypeTree):
        assert self.visit(tree.children[0]) == self.flt
        return int32

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


class FastAnnotateScope(AnnotateScope):
    flt = flt32