from typing import Dict

from lark.visitors import Interpreter
from llvmlite import ir

from util import TypeTree, h_bool, h_int, LambdaAnnotation, target_data, h_byte


def math(self, tree):
    ret = self.visit_children(tree)
    assert ret[0] == ret[1]
    return ret[0]


def comp(self, tree):
    ret = self.visit_children(tree)
    assert ret[0] == ret[1]
    return h_bool


class AnnotateScope(Interpreter):
    def __init__(self, scope: Dict, tree: TypeTree, env):
        self.scope = scope
        self.ret = None
        self.env = env
        self.spots = []
        if tree.data == "code":
            values = self.visit_children(tree)  # sets self.ret
            assert all([v == h_bool for v in values])
            tree.ret = self.ret or h_bool
        else:
            self.visit(tree)  # sets tree.ret

    def visit(self, tree: TypeTree):
        tree.ret = getattr(self, tree.data)(tree)
        return tree.ret

    # def code(self, tree: TypeTree):
    #     return AnnotateScope(self.scope, tree, self.env).ret

    def func(self, tree: TypeTree):
        if tree.children[0].data == "symbol":
            args = {tree.children[0].children[0].value: h_int}
        else:
            args = {a.children[0].value: h_int for a in tree.children[0].children}
        new_scope = self.scope.copy()
        new_scope.update(args)

        keys = tuple(self.scope.keys())
        values = tuple(self.scope.values())
        env = ir.LiteralStructType(values)
        annotation = AnnotateScope(new_scope, tree.children[1], env)

        return LambdaAnnotation(tree.children[1].ret, args.values(), env, keys, annotation.spots)

    def func_call(self, tree):
        args = self.visit(tree.children[1])
        if not isinstance(args, tuple):
            args = (args,)
        func = self.scope[tree.children[0].value]
        tree.func = func
        assert len(args) == len(func.args)

        value = func.return_type

        if isinstance(value, ir.PointerType):
            value = LambdaAnnotation(value.pointee.return_type, value.pointee.args[2:], ir.ArrayType(h_byte, func.spot_size), (), [])

        return value

    def returnn(self, tree: TypeTree):
        self.ret = self.visit(tree.children[0])
        if isinstance(self.ret, LambdaAnnotation):
            # s.env.get_abi_size(target_data)
            self.spots.append(self.ret.env)
            self.ret = self.ret.elements[0]
            # self.spots.append(get_spot_size(self.ret))
        return h_bool

    def symbol(self, tree: TypeTree):
        return self.scope[tree.children[0].value]

    def assign(self, tree: TypeTree):
        name = tree.children[0].children[0].value
        self.scope[name] = self.visit(tree.children[1])
        return h_bool

    def number(self, tree):
        return h_int

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
