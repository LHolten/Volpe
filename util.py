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
