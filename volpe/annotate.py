from typing import Callable

from lark.visitors import Interpreter
from lark import Token
from unification import var, unify, reify

from annotate_utils import logic, unary_logic, math, unary_math, math_assign, comp, shape
from tree import TypeTree, volpe_assert
from volpe_types import (
    int64,
    flt64,
    char,
    VolpeObject,
    VolpeClosure,
    VolpeArray,
    int1,
    Referable
)


class AnnotateScope(Interpreter):
    def __init__(self, tree: TypeTree, scope: Callable, rules: dict, args=None):
        self.scope = scope
        self.local_scope = dict()
        self.used = set()
        self.rules = rules

        if args is not None:
            shape(self, self.local_scope, args, True)

        tree.children[-1] = TypeTree("return_n", [tree.children[-1]], tree.meta)
        tree.return_type = var()

        def ret(value_type):
            volpe_assert(self.unify(value_type, Referable(var(), var(), False)), "cannot return poisoned value", tree)
            volpe_assert(self.unify(tree.return_type, value_type), "block has different return types", tree)
        self.ret = ret

        self.visit_children(tree)  # sets tree.return_type

    def unify(self, a, b):
        self.rules = unify(a, b, self.rules)
        return self.rules is not False

    def visit(self, tree: TypeTree):
        tree.return_type = getattr(self, tree.data)(tree)
        return tree.return_type

    def get_scope(self, name, tree: TypeTree, mut: bool):
        if name in self.local_scope:
            value = self.local_scope[name]
            volpe_assert(value not in self.used, "this value has already been used", tree)
            if mut:
                self.used.add(value)
            return value
        return self.scope(name, tree, mut)

    def symbol(self, tree: TypeTree):
        volpe_type, linear = var(), var()
        self.unify(self.get_scope(tree.children[0].value, tree, False), Referable(volpe_type, linear))
        return Referable(volpe_type, linear, linear)

    def linear(self, tree: TypeTree):
        value = self.get_scope(tree.children[0].value, tree, True)
        volpe_assert(self.unify(value, Referable(linear=True, poison=False)), "var is not linear", tree)
        return value

    def block(self, tree: TypeTree):
        self.rules = AnnotateScope(tree, self.get_scope, self.rules, var()).rules
        return tree.return_type

    def object(self, tree: TypeTree):
        scope = dict()
        poisoned = var()
        for i, child in enumerate(tree.children):
            scope[f"_{i}"] = self.visit(child)
            volpe_assert(self.unify(scope[f"_{i}"], Referable(var(), var(), poisoned)),
                         "all attributes must have the same poison", child)
        return Referable(VolpeObject(scope), True, poisoned)

    def func(self, tree: TypeTree):
        closure = Referable(VolpeClosure(tree=tree), True)
        tree.outside_used = set()

        def scope(name, t_tree: TypeTree, own):
            if name == "@":
                return closure
            tree.outside_used.add(name)
            value = self.get_scope(name, t_tree, own)
            volpe_assert(self.unify(value, Referable(poison=closure.poison)),
                         "all captures must have the same poison", t_tree)
            return value

        self.rules = AnnotateScope(tree.children[1], scope, self.rules, tree.children[0]).rules

        outside_types = {k: self.get_scope(k, tree, False) for k in tree.outside_used}
        self.unify(closure, Referable(
            VolpeClosure(arg=tree.children[0].return_type, ret=tree.children[1].return_type, env=outside_types)))

        return closure

    def func_call(self, tree: TypeTree):
        closure, args = self.visit_children(tree)
        arg_type, ret_type = var(), var()
        volpe_assert(self.unify(closure, Referable(VolpeClosure(arg=arg_type, ret=ret_type))),
                     "can only call closures", tree)
        rules = unify(args, arg_type, self.rules)
        volpe_assert(rules is not False, "wrong arguments for function", tree)
        return reify(ret_type, rules)

    def return_n(self, tree: TypeTree):
        self.ret(self.visit(tree.children[0]))
        return Referable(int1, False, False)

    def assign(self, tree: TypeTree):
        value = self.visit(tree.children[1])
        volpe_assert(self.unify(value, shape(self, self.local_scope, tree.children[0])),
                     "assign error (cannot deconstruct, or tried to assign a ref)", tree)
        return Referable(int1, False, False)

    @staticmethod
    def integer(_: TypeTree):
        return Referable(int64, False, False)

    @staticmethod
    def character(_: TypeTree):
        return Referable(char, False, False)

    @staticmethod
    def escaped_character(_: TypeTree):
        return Referable(char, False, False)

    def string(self, tree: TypeTree):
        tree.data = "list"
        text = eval(tree.children[0])
        tree.children = []
        for eval_character in text:
            tree.children.append(TypeTree("character", [Token("CHARACTER", "'" + eval_character + "'")], tree.meta))
        self.visit_children(tree)
        return Referable(VolpeArray(char), True, False)

    def list_index(self, tree: TypeTree):
        volpe_list, index = self.visit_children(tree)
        volpe_type, linear = var(), var()
        volpe_assert(self.unify(volpe_list, Referable(VolpeArray(Referable(volpe_type, linear)))),
                     "can only index lists", tree)
        volpe_assert(self.unify(index, Referable(int64, False, False)), "can only index with an integer", tree)
        return Referable(volpe_type, linear, linear)

    def list_size(self, tree: TypeTree):
        ret = self.visit_children(tree)[0]
        volpe_assert(self.unify(ret, Referable(VolpeArray(var()))), "can only get size of lists", tree)
        return Referable(int64, False, False)

    def list(self, tree: TypeTree):
        ret = self.visit_children(tree)
        element_type = Referable()
        for value_type in ret:
            volpe_assert(self.unify(element_type, value_type),
                         "different types in list", tree)
        return Referable(VolpeArray(element_type), True, element_type.poison)

    def convert_int(self, tree: TypeTree):
        volpe_assert(self.unify(self.visit(tree.children[0]), Referable(int64)),
                     "can only convert int", tree)
        return Referable(flt64, False, False)

    def convert_flt(self, tree: TypeTree):
        volpe_assert(self.unify(self.visit(tree.children[0]), Referable(flt64)),
                     "can only convert float", tree)
        return Referable(int64, False, False)

    def if_then(self, tree: TypeTree):
        tree.data = "implication"
        old_used = self.used
        self.used = self.used.copy()

        tree.children[1] = TypeTree("return_n", [tree.children[1]], tree.meta)
        value = self.visit(tree)

        self.used = old_used
        return value

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
