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

target_data = llvm.Target.from_default_triple().create_target_machine().target_data


def make_int(n):
    return ir.Constant(int32, n)


def make_flt(n):
    return ir.Constant(flt32, n)


def make_bool(n):
    return ir.Constant(int1, n)


class VolpeTuple(ir.LiteralStructType):
    pass
