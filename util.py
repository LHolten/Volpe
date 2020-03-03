from lark import Tree
from llvmlite import ir

h_bool = ir.IntType(1)
h_int = ir.IntType(32)


class TypeTree(Tree):
    ret = None

    def _pretty_label(self):
        if self.ret is not None:
            return f'{self.data}: {self.ret}'
        return self.data


class Lambda:
    def __init__(self, fntp, keys, arg_keys):
        self.fntp = fntp
        self.func = None
        self.keys = keys
        self.scope = None
        self.arg_keys = arg_keys  # order of the real arguments
