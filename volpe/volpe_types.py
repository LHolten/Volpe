from dataclasses import dataclass
from typing import Dict, Union

import llvmlite.binding as llvm
from llvmlite import ir
from unification import unifiable, Var

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


class VolpeType:
    def unwrap(self) -> ir.Type:
        raise NotImplementedError()


def unwrap(value: Union[ir.Type, VolpeType]) -> ir.Type:
    if isinstance(value, VolpeType):
        return value.unwrap()
    if isinstance(value, Var):
        return int64
    return value


def size(value: Union[ir.Type, VolpeType]) -> ir.Value:
    if isinstance(value, VolpeType):
        value = value.unwrap()
    return int64(value.get_abi_size(target_data))


@unifiable
@dataclass
class VolpeObject(VolpeType):
    type_dict: Dict[str, Union[ir.Type, VolpeType]]

    class Type(ir.LiteralStructType):
        pass

    def __repr__(self):
        return "{" + ", ".join(str(v) for v in self.type_dict.values()) + "}"

    def unwrap(self) -> ir.Type:
        return self.Type(unwrap(value) for value in self.type_dict.values())


@unifiable
@dataclass
class VolpeArray(VolpeType):
    element_type: Union[ir.Type, VolpeType]

    class Type(ir.LiteralStructType):
        pass

    def __repr__(self):
        return f"[{self.element_type}]"

    def unwrap(self) -> ir.Type:
        return self.Type([unwrap(self.element_type).as_pointer(), int64])


@unifiable
@dataclass
class VolpeClosure(VolpeType):
    arg_type: Union[ir.Type, VolpeType]
    ret_type: Union[ir.Type, VolpeType]

    class Type(ir.LiteralStructType):
        pass

    def __repr__(self):
        return f"({self.arg_type})" + "{" + str(self.ret_type) + "}"

    def unwrap(self) -> ir.Type:
        func = ir.FunctionType(unwrap(self.ret_type), [pint8, unwrap(self.arg_type)])
        return self.Type([pint8, func.as_pointer(), free_func.as_pointer()])


@unifiable
@dataclass
class Referable(VolpeType):
    volpe_type: Union[ir.Type, VolpeType]
    is_linear: bool = False
    is_poisoned: bool = False

    def __repr__(self):
        prefix = "&" if self.is_linear else ""
        if isinstance(self.is_linear, Var):
            prefix = "?"
        return f"{prefix}{self.volpe_type}"

    def unwrap(self) -> ir.Type:
        return unwrap(self.volpe_type)

    def __hash__(self):
        return hash(self.__repr__())
