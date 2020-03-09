from typing import Dict

from lark.visitors import Interpreter
from llvmlite import ir

from builder_utils import Closure
from util import TypeTree, int1, int32, pint8, flt32


def logic(self, tree: TypeTree):
    ret = self.visit_children(tree)
    assert ret[0] == int1
    assert ret[1] == int1
    return int1


def unary_logic(self, tree: TypeTree):
    ret = self.visit_children(tree)[0]
    assert ret == int1
    return ret


def math(self, tree: TypeTree):
    ret = self.visit_children(tree)
    assert ret[0] == int32
    assert ret[1] == int32
    return int32


def unary_math(self, tree: TypeTree):
    ret = self.visit_children(tree)[0]
    assert ret == int32
    return ret


def comp(self, tree: TypeTree):
    ret = self.visit_children(tree)
    assert ret[0] == int32
    assert ret[1] == int32
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
    def __init__(self, scope: Dict, tree: TypeTree, closure: Unannotated, is_func):
        self.scope = scope
        self.tree = tree
        self.closure = closure
        self.is_func = is_func

        if tree.data == "code":
            values = self.visit_children(tree)  # sets self.ret
            assert all([v == int1 for v in values]), "some line does not evaluate to a bool"
        else:
            self.update_return(self.visit(tree))

        assert self.tree.ret, "void methods should return true"

    def update_return(self, ret):
        if self.tree.ret is not None:
            assert self.tree.ret == ret, "different return types encountered in same block"
        self.tree.ret = ret
        if self.is_func:
            func = ir.FunctionType(ret, self.closure.func.args)
            self.closure.update(func)

    def visit(self, tree: TypeTree):
        tree.ret = getattr(self, tree.data)(tree)
        return tree.ret

    def code(self, tree: TypeTree):
        AnnotateScope(self.scope.copy(), tree, self.closure, False)
        return tree.ret

    def func(self, tree: TypeTree):
        new_scope = self.scope.copy()

        if tree.children[0].data == "symbol":
            arg_names = [tree.children[0].children[0].value]
        else:
            arg_names = [a.children[0].value for a in tree.children[0].children]

        return Unannotated(new_scope, arg_names, tree.children[1])

    def func_call(self, tree: TypeTree) -> ir.Type:
        closure, arg_types = self.visit_children(tree)

        assert isinstance(closure, Unannotated)

        if closure.checked:  # we have already been here
            return closure.func.return_type
        closure.checked = True

        if not isinstance(arg_types, tuple):
            arg_types = (arg_types,)

        scope = closure.scope
        scope.update(dict(zip(closure.arg_names, arg_types)))

        closure.update(ir.FunctionType(ir.VoidType(), [pint8, *arg_types]))

        AnnotateScope(scope, closure.code, closure, True)

        return closure.func.return_type

    def this_func(self, tree: TypeTree):
        return self.closure

    def returnn(self, tree: TypeTree):
        self.update_return(self.visit(tree.children[0]))
        return int1

    def symbol(self, tree: TypeTree):
        return self.scope[tree.children[0].value]

    def assign(self, tree: TypeTree):
        name = tree.children[0].children[0].value
        self.scope[name] = self.visit(tree.children[1])
        return int1

    def number(self, tree: TypeTree):
        return int32

    def tuple(self, tree: TypeTree):
        return tuple(self.visit_children(tree))

    implication = logic
    logic_and = logic
    logic_or = logic
    logic_not = unary_logic

    add = math
    mod = math
    mul = math
    sub = math
    div = math
    pow = math
    negate = unary_math

    equals = comp
    not_equals = comp
    greater = comp
    less = comp
    greater_equals = comp
    less_equals = comp

    def __default__(self, tree: TypeTree):
        raise NotImplementedError("annotate", tree.data)
