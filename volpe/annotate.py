from typing import Callable

from lark.visitors import Interpreter
from lark import Token
from unification import var, unify

from annotate_utils import logic, unary_logic, math, unary_math, math_assign, comp, shape
from tree import TypeTree
from volpe_types import (
    int64,
    flt64,
    char,
    VolpeObject,
    VolpeClosure,
    VolpeList, int1
)


class AnnotateScope(Interpreter):
    def __init__(self, tree: TypeTree, scope: Callable, rules: dict, return_type, args=None):
        self.scope = scope
        self.local_scope = dict()
        self.rules = rules

        if args is not None:
            self.unify(args[0], shape(self, self.local_scope, args[1]))

        tree.children[-1] = TypeTree("return_n", [tree.children[-1]])
        tree.return_type = return_type

        def ret(value_type):
            assert self.unify(tree.return_type, value_type), "block has different return types"
        self.ret = ret

        self.visit_children(tree)  # sets tree.return_type

    def unify(self, a, b):
        self.rules = unify(a, b, self.rules)
        return self.rules is not False

    def visit(self, tree: TypeTree):
        tree.return_type = getattr(self, tree.data)(tree)
        return tree.return_type

    def get_scope(self, name):
        if name in self.local_scope:
            return self.local_scope[name]
        return self.scope(name)

    def block(self, tree: TypeTree):
        self.rules = AnnotateScope(tree, self.get_scope, self.rules, var()).rules
        return tree.return_type

    def object(self, tree: TypeTree):
        scope = dict()
        for i, child in enumerate(tree.children):
            name = f"_{i}"
            scope[name] = self.visit(child)
        return VolpeObject(scope)

    def func(self, tree: TypeTree):
        closure = VolpeClosure(var(), var())

        tree.outside_used = set()

        def scope(name):
            if name == "@":
                return closure
            tree.outside_used.add(name)
            return self.get_scope(name)

        self.rules = AnnotateScope(tree.children[1], scope, self.rules, closure.ret_type,
                                   (closure.arg_type, tree.children[0])).rules
        return closure

    def func_call(self, tree: TypeTree):
        closure = self.visit(tree.children[0])
        arg_type, ret_type = var(), var()
        assert self.unify(closure, VolpeClosure(arg_type, ret_type)), "can only call closures"
        new_arg_type = self.visit(tree.children[1])
        assert self.unify(new_arg_type, arg_type), "wrong argument type"
        return ret_type

    def return_n(self, tree: TypeTree):
        self.ret(self.visit(tree.children[0]))
        return int1

    def symbol(self, tree: TypeTree):
        return self.get_scope(tree.children[0].value)

    def assign(self, tree: TypeTree):
        assert self.unify(shape(self, self.local_scope, tree.children[0]), self.visit(tree.children[1])), "assign error"
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

    def string(self, tree: TypeTree):
        tree.data = "list"
        text = eval(tree.children[0])
        tree.children = []
        for eval_character in text:
            tree.children.append(TypeTree("character", [Token("CHARACTER", "'" + eval_character + "'")]))
        self.visit_children(tree)
        return VolpeList(char)

    def list_index(self, tree: TypeTree):
        ret = self.visit_children(tree)
        element_type = var()
        assert self.unify(ret[0], VolpeList(element_type)), "can only index lists"
        assert self.unify(ret[1], int64), "can only index with an integer"
        return element_type

    def list_size(self, tree: TypeTree):
        ret = self.visit_children(tree)[0]
        assert self.unify(ret, VolpeList(var())), "can only get size of lists"
        return int64

    def list(self, tree: TypeTree):
        ret = self.visit_children(tree)
        element_type = var()
        for value_type in ret:
            assert self.unify(element_type, value_type), "different types in list"
        return VolpeList(element_type)

    def convert_int(self, tree: TypeTree):
        assert self.unify(self.visit(tree.children[0]), int64), "can only convert int"
        return flt64

    def convert_flt(self, tree: TypeTree):
        assert self.unify(self.visit(tree.children[0]), flt64), "can only convert float"
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
