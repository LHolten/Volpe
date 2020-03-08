from typing import Dict

from lark.visitors import Interpreter
from llvmlite import ir

from builder_utils import Closure
from util import TypeTree, int1, int32, pint8


def math(self, tree):
    ret = self.visit_children(tree)
    assert ret[0] == ret[1]
    return ret[0]


def comp(self, tree):
    ret = self.visit_children(tree)
    assert ret[0] == ret[1]
    return int1


class Unannotated(Closure):
    def __init__(self, scope, arg_names, code):
        super().__init__(ir.FunctionType(ir.VoidType(), []).as_pointer())
        self.scope = scope
        self.arg_names = arg_names
        self.code = code
        self.checked = False

    def update(self, func: ir.FunctionType):
        super().__init__(func.as_pointer())


class AnnotateScope(Interpreter):
    def __init__(self, scope: Dict, tree: TypeTree):
        self.scope = scope
        self.ret = None

        if tree.data == "code":
            values = self.visit_children(tree)  # sets self.ret
            assert all([v == int1 for v in values]), "some line does not evaluate to a bool"
            assert self.ret, "void methods should return true"
            tree.ret = self.ret
        else:
            self.visit(tree)  # sets tree.ret

    def visit(self, tree: TypeTree):
        tree.ret = getattr(self, tree.data)(tree)
        return tree.ret

    def code(self, tree: TypeTree):
        return AnnotateScope(self.scope.copy(), tree).ret

    def func(self, tree: TypeTree):
        new_scope = self.scope.copy()

        if tree.children[0].data == "symbol":
            arg_names = [tree.children[0].children[0].value]
        else:
            arg_names = [a.children[0].value for a in tree.children[0].children]

        return Unannotated(new_scope, arg_names, tree.children[1])

    def func_call(self, tree: TypeTree) -> ir.Type:
        func_tree, arg_tree = tree.children

        closure = self.visit(func_tree)
        assert isinstance(closure, Unannotated)

        if closure.checked:  # we have already been here
            return closure.func.return_type
        closure.checked = True

        arg_types = self.visit(arg_tree)
        if not isinstance(arg_types, tuple):
            arg_types = (arg_types,)

        scope = closure.scope
        scope.update(dict(zip(closure.arg_names, arg_types)))

        AnnotateScope(scope, closure.code)
        return_type = closure.code.ret

        func = ir.FunctionType(return_type, [pint8, *arg_types])

        closure.update(func)

        return return_type

    def returnn(self, tree: TypeTree):
        ret = self.visit(tree.children[0])
        if not isinstance(ret, Unannotated) or self.ret is None:  # help out type inference
            self.ret = ret
        return int1

    def symbol(self, tree: TypeTree):
        return self.scope[tree.children[0].value]

    def assign(self, tree: TypeTree):
        name = tree.children[0].children[0].value
        self.scope[name] = self.visit(tree.children[1])
        return int1

    def implication(self, tree: TypeTree):
        ret = self.visit_children(tree)
        assert ret[0] == int1
        assert ret[1] == int1
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
