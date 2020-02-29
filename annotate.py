from typing import Dict

from lark.visitors import Interpreter
from llvmlite import ir

from util import TypeTree, h_bool, h_int


def math(self, tree):
    ret = self.visit_children(tree)
    assert ret[0] == ret[1]
    return ret[0]


def comp(self, tree):
    ret = self.visit_children(tree)
    assert ret[0] == ret[1]
    return h_bool


class AnnotateScope(Interpreter):
    def __init__(self, scope: Dict, args: Dict, tree: TypeTree):
        self.scope = scope.copy()
        self.scope.update(args)
        self.ret = None
        if tree.data == "code":
            values = self.visit_children(tree)
            assert all([v == h_bool for v in values])
            tree.ret = self.ret or h_bool
        else:
            self.visit(tree)

    def visit(self, tree: TypeTree):
        tree.ret = getattr(self, tree.data)(tree)
        return tree.ret

    def code(self, tree: TypeTree):
        return AnnotateScope(self.scope, {}, tree).ret

    def func(self, tree: TypeTree):
        if tree.children[0].data == "symbol":
            args = {tree.children[0].children[0].value: h_int}
        else:
            args = {a.children[0].value: h_int for a in tree.children[0].children}
        AnnotateScope(self.scope, args, tree.children[1])
        return ir.FunctionType(tree.children[1].ret, args.values())

    def symbol(self, tree: TypeTree):
        return self.scope[tree.children[0].value]

    def assign(self, tree: TypeTree):
        name = tree.children[0].children[0].value
        self.scope[name] = self.visit(tree.children[1])
        return h_bool

    def number(self, tree):
        return h_int

    def returnn(self, tree: TypeTree):
        self.ret = self.visit(tree.children[0])
        return h_bool

    def func_call(self, tree):
        self.visit(tree.children[1])
        return self.scope[tree.children[0].value].return_type

    def tuple(self, tree):
        return tuple(self.visit_children(tree))

    add = math
    mod = math
    mul = math
    sub = math
    div = math
    pow = math

    equals = comp
    not_equals = comp
    greater = comp
    less = comp
    greater_equals = comp
    less_equals = comp

    def __default__(self, tree):
        raise NotImplementedError("annotate", tree.data)
