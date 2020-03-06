from typing import Dict, List

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
    def __init__(self, scope, tree, arg_names, code):
        super().__init__(ir.FunctionType(ir.VoidType(), []).as_pointer())
        self.scope = scope
        self.tree: TypeTree = tree  # this is the one that ultimately needs updating
        self.arg_names = arg_names
        self.code = code
        self.return_of: List[Unannotated] = []
        self.arg_of: List[(int, Unannotated)] = []

    def update(self, func: ir.FunctionType):
        super().__init__(func.as_pointer())

        for r in self.return_of:
            arg_types = r.func.args
            new_func = ir.FunctionType(self, arg_types)
            r.update(new_func)

        for i, a in self.arg_of:
            return_type = a.func.return_type
            arg_types = list(a.func.args)
            arg_types[i + 1] = self
            new_func = ir.FunctionType(return_type, arg_types)
            a.update(new_func)


class AnnotateScope(Interpreter):
    def __init__(self, scope: Dict, tree: TypeTree):
        self.scope = scope
        self.ret = None

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

    def func(self, tree: TypeTree):
        new_scope = self.scope.copy()

        if tree.children[0].data == "symbol":
            arg_names = [tree.children[0].children[0].value]
        else:
            arg_names = [a.children[0].value for a in tree.children[0].children]

        return Unannotated(new_scope, tree, arg_names, tree.children[1])

    def func_call(self, tree: TypeTree) -> ir.Type:
        func_name, arg_tree = tree.children

        arg_types = self.visit(arg_tree)  # these can be Unannotated
        if not isinstance(arg_types, tuple):
            arg_types = (arg_types,)

        closure = self.scope[func_name.value]

        for i, a in enumerate(arg_types):
            if isinstance(a, Unannotated):
                a.arg_of.append((i, closure))

        assert isinstance(closure, Unannotated)

        scope = closure.scope
        scope.update(dict(zip(closure.arg_names, arg_types)))

        AnnotateScope(scope, closure.code)
        # now all arguments should be annotated, as well as the return type
        return_type = closure.code.ret

        if isinstance(return_type, Unannotated):
            return_type.return_of.append(closure)
            # create temp new closure
            func = ir.FunctionType(ir.VoidType(), [pint8, *arg_types])
        else:
            # create actual closure
            func = ir.FunctionType(return_type, [pint8, *arg_types])

        closure.update(func)

        return return_type

    def returnn(self, tree: TypeTree):
        self.ret = self.visit(tree.children[0])
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
