from typing import Callable

from lark.visitors import Interpreter

from annotate_utils import logic, unary_logic, math, unary_math, math_assign, comp
from tree import TypeTree
from volpe_types import (
    int1,
    int64,
    flt64,
    char,
    VolpeObject,
    VolpeClosure,
    VolpeList, combine_types, tuple_assign, VolpeBlock
)


class AnnotateScope(Interpreter):
    def __init__(self, tree: TypeTree, instance_scope: Callable, ret: Callable):
        self.instance_scope = instance_scope
        self.local_scope = dict()
        self.ret = ret

        if tree.children[-1].data != "return_n":
            tree.children[-1] = TypeTree("return_n", [tree.children[-1]])

        values = self.visit_children(tree)  # sets tree.return_type
        assert all([v == int1 for v in values]), "some line does not evaluate to a bool"

    def visit(self, tree: TypeTree):
        tree.return_type = getattr(self, tree.data)(tree)
        return tree.return_type

    def get_instance_scope(self):
        frozen_local = self.local_scope.copy()
        frozen_parent = self.instance_scope()

        def scope(name):
            if name in frozen_local:
                return frozen_local[name]
            else:
                return frozen_parent(name)

        return scope

    def block(self, tree: TypeTree):
        def ret(value_type):
            if tree.return_type is not None:
                combine_types(self, value_type, tree.return_type)
            tree.return_type = value_type

        self.__class__(tree, self.get_instance_scope, ret)
        return tree.return_type

    def object(self, tree: TypeTree):
        scope = dict()
        for i, child in enumerate(tree.children):
            name = f"_{i}"
            scope[name] = self.visit(child)
        res = VolpeObject(scope)
        return res

    def func(self, tree: TypeTree):
        tree.block = VolpeBlock(tree, self.get_instance_scope)
        return VolpeClosure(tree.block)

    def func_call(self, tree: TypeTree):
        closure = self.visit(tree.children[0])
        arg_type = self.visit(tree.children[1])

        assert isinstance(closure, VolpeClosure), "can only call closures"

        return closure.call(arg_type, self.__class__)

    def return_n(self, tree: TypeTree):
        self.ret(self.visit(tree.children[0]))
        return int1

    def symbol(self, tree: TypeTree):
        return self.get_instance_scope()(tree.children[0].value)

    def assign(self, tree: TypeTree):
        tuple_assign(self.local_scope, tree.children[0], self.visit(tree.children[1]))
        return int1

    @staticmethod
    def integer(tree: TypeTree):
        return int64

    @staticmethod
    def character(tree: TypeTree):
        return char

    @staticmethod
    def escaped_character(tree: TypeTree):
        return char

    def list_index(self, tree: TypeTree):
        ret = self.visit_children(tree)
        assert isinstance(ret[0], VolpeList)
        assert ret[1] == int64
        return ret[0].element_type

    def list_size(self, tree: TypeTree):
        ret = self.visit_children(tree)[0]
        assert isinstance(ret, VolpeList)
        return int64

    def list(self, tree: TypeTree):
        ret = combine_types(self, self.visit_children(tree))
        return VolpeList(ret)

    def convert_int(self, tree: TypeTree):
        assert self.visit(tree.children[0]) == int64
        return flt64

    def convert_flt(self, tree: TypeTree):
        assert self.visit(tree.children[0]) == flt64
        return int64

    def if_then(self, tree: TypeTree):
        tree.data = "implication"
        tree.children[1] = TypeTree("return_n", [tree.children[1]])
        return self.visit(tree)

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
