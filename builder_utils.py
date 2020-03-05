from typing import List

from llvmlite import ir

from util import h, pint8, int32, target_data, h_b


class ClosurePointer(ir.LiteralStructType):
    reservation: int = None
    size: int = None

    def __init__(self, func_ptr: ir.PointerType):
        super().__init__([func_ptr, int32, int32, pint8])
        self.func: ir.FunctionType = func_ptr.pointee

    @classmethod
    def from_closure(cls, other: 'Closure', size: int):
        ret = cls(other.elements[0])
        ret.reservation = other.reservation
        ret.size = size
        return ret


class Closure(ir.LiteralStructType):
    reservation = None

    def __init__(self, func_ptr: ir.PointerType):
        super().__init__([func_ptr, int32])
        self.func: ir.FunctionType = func_ptr.pointee

    @classmethod
    def from_closure_pointer(cls, other: ClosurePointer):
        ret = cls(other.elements[0])
        ret.reservation = other.reservation
        return ret


def scope_size(type_list: List[ir.Type]) -> int:
    total = 0

    for t in type_list:
        total += t.get_abi_size(target_data)
        if isinstance(t, ClosurePointer):
            total += t.size + t.get_abi_size(target_data) - pint8.get_abi_size(target_data)

    return total


def write_environment(b: ir.IRBuilder, ptr: ir.NamedValue, value_list: List[ir.NamedValue]) -> None:
    memcpy = b.module.declare_intrinsic('llvm.memcpy', [pint8, pint8, int32])

    for value in value_list:
        ptr = b.bitcast(ptr, value.type.as_pointer())
        b.store(value, ptr)
        if isinstance(value.type, ClosurePointer):
            env_size = b.extract_value(value, 2)
            env_pointer = b.extract_value(value, 3)
            ptr = b.gep(ptr, (h(0), h(3)))  # move to where the environment should be copied
            ptr = b.bitcast(ptr, pint8)
            b.call(memcpy, [ptr, env_pointer, env_size, h_b(0)])
            ptr = b.gep(ptr, (env_size,))  # move to the end of the environment
        else:
            ptr = b.gep(ptr, (h(1),))


def read_environment(b: ir.IRBuilder, ptr: ir.NamedValue, type_list: List[ir.Type]) -> List[ir.NamedValue]:
    value_list = []

    for t in type_list:
        ptr = b.bitcast(ptr, t.as_pointer())
        value = b.load(ptr)
        if isinstance(value.type, ClosurePointer):
            env_size = b.extract_value(value, 2)
            ptr = b.gep(ptr, (h(0), h(3)))  # move to where the environment should referenced
            ptr = b.bitcast(ptr, pint8)
            value = b.insert_value(value, ptr, 3)
            ptr = b.gep(ptr, (env_size,))
        else:
            ptr = b.gep(ptr, (h(1),))

        value_list.append(value)

    return value_list
