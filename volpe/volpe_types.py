from typing import Dict

import llvmlite.binding as llvm
from llvmlite import ir

llvm.initialize()
llvm.initialize_native_target()
llvm.initialize_native_asmprinter()  # yes, even this one

int1 = ir.IntType(1)
int32 = ir.IntType(32)
int8 = ir.IntType(8)
pint8 = int8.as_pointer()
flt32 = ir.FloatType()
flt64 = ir.DoubleType()
unknown = ir.VoidType()
copy_func = ir.FunctionType(pint8, [pint8])
free_func = ir.FunctionType(unknown, [pint8])
unknown_func = ir.FunctionType(unknown, [])

target_data = llvm.Target.from_default_triple().create_target_machine().target_data


class VolpeTuple(ir.LiteralStructType):
    pass


class Closure(ir.LiteralStructType):
    def __init__(self, outside_scope: Dict, arg_names, block):
        super().__init__([unknown_func.as_pointer(), copy_func.as_pointer(), free_func.as_pointer(), pint8])
        self.func = unknown_func
        self.outside_scope = outside_scope
        self.outside_used = set()
        self.scope = dict()
        self.arg_names = arg_names
        self.block = block
        self.checked = False

    def update(self, func: ir.FunctionType):
        super().__init__([func.as_pointer(), copy_func.as_pointer(), free_func.as_pointer(), pint8])
        self.func = func
