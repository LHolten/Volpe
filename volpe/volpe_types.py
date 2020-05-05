from typing import Dict, Callable

import llvmlite.binding as llvm
from llvmlite import ir

llvm.initialize()
llvm.initialize_native_target()
llvm.initialize_native_asmprinter()  # yes, even this one

int1 = ir.IntType(1)
int32 = ir.IntType(32)
int64 = ir.IntType(64)
int8 = ir.IntType(8)
pint8 = int8.as_pointer()
# flt32 = ir.FloatType()
flt64 = ir.DoubleType()
char = ir.IntType(8)
unknown = ir.VoidType()
copy_func = ir.FunctionType(pint8, [pint8])
free_func = ir.FunctionType(unknown, [pint8])
unknown_func = ir.FunctionType(unknown, [pint8, pint8])

target_data = llvm.Target.from_default_triple().create_target_machine().target_data


class VolpeObject(ir.LiteralStructType):
    def __init__(self, type_dict: Dict[str, ir.Type]):
        super().__init__(type_dict.values())
        self.type_dict = type_dict

    def set(self, name: str, t: ir.Type):
        self.type_dict[name] = t
        self.__init__(self.type_dict)


class VolpeList(ir.LiteralStructType):
    def __init__(self, element_type: ir.Type):
        super().__init__([element_type.as_pointer(), int32])
        self.element_type = element_type


class VolpeClosure(ir.LiteralStructType):
    def __init__(self, scope: Callable, local_scope: dict, arg_object, block):
        super().__init__([unknown_func.as_pointer(), copy_func.as_pointer(), free_func.as_pointer(), pint8])
        self.func = unknown_func
        self.outside_used = set()
        self.arg_object = arg_object
        self.block = block
        self.checked = False

        frozen_scope = local_scope.copy()

        def get_scope(name):
            self.outside_used.add(name)
            if name in frozen_scope:
                return frozen_scope[name]
            else:
                return scope(name)
        self.get_scope = get_scope

    def update(self, func: ir.FunctionType):
        super().__init__([func.as_pointer(), copy_func.as_pointer(), free_func.as_pointer(), pint8])
        self.func = func
