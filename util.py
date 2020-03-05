from typing import List

import llvmlite.binding as llvm
from lark import Tree
from llvmlite import ir

llvm.initialize()
llvm.initialize_native_target()
llvm.initialize_native_asmprinter()  # yes, even this one

int1 = ir.IntType(1)
int32 = ir.IntType(32)
int8 = ir.IntType(8)
pint8 = int8.as_pointer()
target_data = llvm.Target.from_default_triple().create_target_machine().target_data


def h(n):
    return ir.Constant(int32, n)


def h_b(n):
    return ir.Constant(int1, n)


class TypeTree(Tree):
    ret = None

    def _pretty_label(self):
        if self.ret is not None:
            return f'{self.data}: {self.ret}'
        return self.data
