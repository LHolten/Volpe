from typing import Dict

from lark.visitors import Interpreter
from llvmlite import ir

from builder_utils import Closure, ClosurePointer, scope_size
from util import TypeTree, int1, int32, pint8


def math(self, tree):
    ret = self.visit_children(tree)
    assert ret[0] == ret[1]
    return ret[0]


def comp(self, tree):
    ret = self.visit_children(tree)
    assert ret[0] == ret[1]
    return int1


class AnnotateScope(Interpreter):
    def __init__(self, scope: Dict, tree: TypeTree):
        self.scope = scope
        self.ret = None
        self.reservation = []

        if tree.data == "code":
            values = self.visit_children(tree)  # sets self.ret
            assert all([v == int1 for v in values])
            tree.ret = self.ret or int1
        else:
            self.visit(tree)  # sets tree.ret

    def visit(self, tree: TypeTree):
        tree.ret = getattr(self, tree.data)(tree)
        return tree.ret

    # def code(self, tree: TypeTree):
    #     return AnnotateScope(self.scope, tree, self.env).ret

    def func(self, tree: TypeTree) -> ClosurePointer:
        if tree.children[0].data == "symbol":
            args = {tree.children[0].children[0].value: int32}
        else:
            args = {a.children[0].value: int32 for a in tree.children[0].children}
        new_scope = self.scope.copy()
        new_scope.update(args)

        type_list = list(self.scope.values())

        annotation = AnnotateScope(new_scope, tree.children[1])

        arg_types = [pint8, pint8, *args.values()]
        return_type = tree.children[1].ret
        closure = ClosurePointer(ir.FunctionType(return_type, arg_types).as_pointer())
        closure.size = scope_size(type_list)
        closure.reservation = max(annotation.reservation) if annotation.reservation else 0

        return closure

    def func_call(self, tree: TypeTree) -> ir.Type:
        args = self.visit(tree.children[1])
        if not isinstance(args, tuple):
            args = (args,)
        closure = self.scope[tree.children[0].value]
        assert isinstance(closure, ClosurePointer)
        assert len(args) == len(closure.func.args) - 2

        t = closure.func.return_type

        if isinstance(t, Closure):
            t = ClosurePointer.from_closure(t, closure.reservation)

        return t

    def returnn(self, tree: TypeTree):
        t = self.visit(tree.children[0])
        if isinstance(t, ClosurePointer):
            self.reservation.append(t.size)
            t = Closure.from_closure_pointer(t)
        self.ret = t
        return int1

    def symbol(self, tree: TypeTree):
        return self.scope[tree.children[0].value]

    def assign(self, tree: TypeTree):
        name = tree.children[0].children[0].value
        self.scope[name] = self.visit(tree.children[1])
        return int1

    def number(self, tree):
        return int32

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
