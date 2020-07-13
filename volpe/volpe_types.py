from dataclasses import dataclass, field
from typing import Dict, Union

import llvmlite.binding as llvm
from llvmlite import ir
from unification import unifiable, Var, var

from tree import TypeTree

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


def vary():
    return field(default_factory=var)


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
    arg: Union[ir.Type, VolpeType] = vary()
    ret: Union[ir.Type, VolpeType] = vary()
    env: Dict[str, Union[ir.Type, VolpeType]] = vary()
    tree: TypeTree = vary()

    class Type(ir.LiteralStructType):
        pass

    def __repr__(self):
        if self.tree is None:
            return "func"
        return f"({self.arg})" + "{" + str(self.ret) + "}"

    def unwrap(self) -> ir.Type:
        return self.Type(unwrap(value) for value in self.type_dict.values())


@unifiable
@dataclass
class Referable(VolpeType):
    volpe_type: Union[ir.Type, VolpeType] = vary()
    linear: bool = vary()
    poison: bool = vary()

    def __repr__(self):
        prefix = "&" if self.linear else ""
        if isinstance(self.linear, Var):
            prefix = "?"
        return f"{prefix}{self.volpe_type}"

    def unwrap(self) -> ir.Type:
        return unwrap(self.volpe_type)

    def __hash__(self):
        return hash(self.__repr__())
