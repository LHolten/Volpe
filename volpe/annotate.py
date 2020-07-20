from typing import Callable

from lark.visitors import Interpreter
from lark import Token
from unification import unify
from copy import deepcopy

from annotate_utils import logic, unary_logic, math, unary_math, math_assign, comp, shape
from tree import TypeTree, volpe_assert
from unification_copy import var
from volpe_types import (
    int64,
    flt64,
    char,
    VolpeObject,
    VolpeClosure,
    VolpeArray,
    int1,
)


class AnnotateScope(Interpreter):
    def __init__(self, tree: TypeTree, scope: Callable, rules: dict, args=None):
        self.scope = scope
        self.local_scope = dict()
        self.used = set()
        self.rules = rules

        if args is not None:
            shape(self, self.local_scope, args)

        tree.children[-1] = TypeTree("return_n", [tree.children[-1]], tree.meta)
        tree.return_type = var()

        def ret(value_type):
            volpe_assert(self.unify(tree.return_type, value_type), "block has different return types", tree)
        self.ret = ret

        self.visit_children(tree)  # sets tree.return_type

    def unify(self, a, b):
        self.rules = unify(a, b, self.rules)
        return self.rules is not False

    def visit(self, tree: TypeTree):
        tree.return_type = getattr(self, tree.data)(tree)
        if tree.return_type is None:
            tree.return_type = int1
        return tree.return_type

    def get_scope(self, name, tree: TypeTree):
        if name in self.local_scope:
            return self.local_scope[name]
        return self.scope(name, tree)

    def symbol(self, tree: TypeTree):
        return self.get_scope(tree.children[0].value, tree)

    def block(self, tree: TypeTree):
        self.rules = AnnotateScope(tree, self.get_scope, self.rules, var()).rules
        return tree.return_type

    def object(self, tree: TypeTree):
        scope = dict()
        for i, child in enumerate(tree.children):
            scope[f"_{i}"] = self.visit(child)
        return VolpeObject(scope)

    def func(self, tree: TypeTree):
        closure = VolpeClosure(tree=tree)
        tree.outside_used = set()

        def scope(name, t_tree: TypeTree):
            if name == "@":
                return closure
            tree.outside_used.add(name)
            return self.get_scope(name, t_tree)

        self.rules = AnnotateScope(tree.children[1], scope, self.rules, tree.children[0]).rules

        outside_types = {k: self.get_scope(k, tree) for k in tree.outside_used}
        assert self.unify(closure, VolpeClosure(env=outside_types))
        assert self.unify(closure, VolpeClosure(arg=tree.children[0].return_type, ret=tree.children[1].return_type))

        return closure

    def func_call(self, tree: TypeTree):
        closure, args = self.visit_children(tree)
        env, ret_type = var(), var()

        if not tree.children[0].data == "symbol" or not tree.children[0].children[0].value == "@":
            volpe_assert(self.unify(closure, VolpeClosure(env=env)), "can only call closures", tree)

            rules, closure = deepcopy((self.rules, closure))
            self.rules.update(rules)

        volpe_assert(self.unify(closure, VolpeClosure(arg=args, ret=ret_type, env=env)),
                     "wrong arguments for function", tree)
        return ret_type

    def return_n(self, tree: TypeTree):
        self.ret(self.visit(tree.children[0]))

    def assign(self, tree: TypeTree):
        value = self.visit(tree.children[1])
        volpe_assert(self.unify(shape(self, self.local_scope, tree.children[0]), value), "assign error", tree)

    @staticmethod
    def integer(_: TypeTree):
        return int64

    @staticmethod
    def character(_: TypeTree):
        return char

    @staticmethod
    def escaped_character(_: TypeTree):
        return char

    def string(self, tree: TypeTree):
        tree.data = "list"
        text = eval(tree.children[0])
        tree.children = []
        for eval_character in text:
            tree.children.append(TypeTree("character", [Token("CHARACTER", "'" + eval_character + "'")], tree.meta))
        self.visit_children(tree)
        return VolpeArray(char)

    def list_index(self, tree: TypeTree):
        volpe_array, index = self.visit_children(tree)
        volpe_type = var()
        volpe_assert(self.unify(volpe_array, VolpeArray(volpe_type)), "can only index arrays", tree)
        volpe_assert(self.unify(index, int64), "can only index with an integer", tree)
        return volpe_type

    def list_size(self, tree: TypeTree):
        volpe_array = self.visit_children(tree)[0]
        volpe_assert(self.unify(volpe_array, VolpeArray()), "can only get size of arrays", tree)
        return int64

    def list(self, tree: TypeTree):
        element_type = var()
        for child in tree.children:
            volpe_assert(self.unify(element_type, self.visit(child)), "different types in list", tree)
        return VolpeArray(element_type, len(tree.children))

    def convert_int(self, tree: TypeTree):
        volpe_assert(self.unify(self.visit(tree.children[0]), int64), "can only convert int", tree)
        return flt64

    def convert_flt(self, tree: TypeTree):
        volpe_assert(self.unify(self.visit(tree.children[0]), flt64), "can only convert float", tree)
        return int64

    def if_then(self, tree: TypeTree):
        tree.data = "implication"
        tree.children[1] = TypeTree("return_n", [tree.children[1]], tree.meta)
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
