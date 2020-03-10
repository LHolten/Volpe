from contextlib import contextmanager
from typing import List, Dict

from llvmlite import ir

from tree import TypeTree
from volpe_types import make_int, pint8, int32, target_data, make_bool, VolpeTuple


class Closure(ir.LiteralStructType):
    def __init__(self, func_ptr: ir.PointerType):
        super().__init__([func_ptr, int32, pint8])
        self.func: ir.FunctionType = func_ptr.pointee


def environment_size(b: ir.IRBuilder, value_list: List) -> int:
    total = make_int(0)

    for value in value_list:
        if isinstance(value.type, Closure):
            total = b.add(total, make_int(value.type.get_abi_size(target_data) - pint8.get_abi_size(target_data)))
            total = b.add(total, b.extract_value(value, 1))
        else:
            total = b.add(total, make_int(value.type.get_abi_size(target_data)))

    return total


def free_environment(b: ir.IRBuilder, value_list: set) -> None:
    for value in value_list:
        if isinstance(value.type, Closure):
            env_ptr = b.extract_value(value, 2)
            b.call(b.module.free, [env_ptr])


def write_environment(b: ir.IRBuilder, ptr: ir.NamedValue, value_list: List) -> None:
    for value in value_list:
        if isinstance(value.type, Closure):
            func_ptr = b.extract_value(value, 0)
            env_size = b.extract_value(value, 1)
            env_pointer = b.extract_value(value, 2)

            # need to store every part manually to not overwrite anything
            ptr = b.bitcast(ptr, value.type.elements[0].as_pointer())
            b.store(func_ptr, ptr)
            ptr = b.gep(ptr, (make_int(1),))
            ptr = b.bitcast(ptr, value.type.elements[1].as_pointer())
            b.store(env_size, ptr)
            ptr = b.gep(ptr, (make_int(1),))
            env_ptr = b.bitcast(ptr, pint8)
            ptr = b.gep(env_ptr, (env_size,))

            b.call(b.module.memcpy, [env_ptr, env_pointer, env_size, make_bool(0)])
        else:
            ptr = b.bitcast(ptr, value.type.as_pointer())
            b.store(value, ptr)
            ptr = b.gep(ptr, (make_int(1),))


def read_environment(b: ir.IRBuilder, ptr: ir.NamedValue, type_list: List) -> List[ir.NamedValue]:
    value_list = []

    for t in type_list:
        if isinstance(t, Closure):
            ptr = b.bitcast(ptr, t.elements[0].as_pointer())
            func_ptr = b.load(ptr)
            ptr = b.gep(ptr, (make_int(1),))
            ptr = b.bitcast(ptr, t.elements[1].as_pointer())
            env_size = b.load(ptr)
            ptr = b.gep(ptr, (make_int(1),))
            env_ptr = b.bitcast(ptr, pint8)
            ptr = b.gep(env_ptr, (env_size,))

            env_pointer = b.call(b.module.malloc, [env_size])
            b.call(b.module.memcpy, [env_pointer, env_ptr, env_size, make_bool(0)])

            value = ir.Constant(t, ir.Undefined)
            value = b.insert_value(value, func_ptr, 0)
            value = b.insert_value(value, env_size, 1)
            value = b.insert_value(value, env_pointer, 2)
        else:
            ptr = b.bitcast(ptr, t.as_pointer())
            value = b.load(ptr)
            ptr = b.gep(ptr, (make_int(1),))

        value_list.append(value)

    return value_list


@contextmanager
def options(b: ir.IRBuilder, t: ir.Type, phi) -> ir.Value:
    new_block = b.function.append_basic_block("block")
    with b.goto_block(new_block):
        phi_node = b.phi(t)

    def ret(value):
        if not b.block.is_terminated:
            phi_node.add_incoming(value, b.block)
            b.branch(new_block)

    yield ret

    b.position_at_end(new_block)
    phi.append(phi_node)


def tuple_assign(scope: Dict, b: ir.IRBuilder, tree: TypeTree, value):
    if tree.data == "collect_tuple":
        for i, child in enumerate(tree.children):
            tuple_assign(scope, b, child, b.extract_value(value, i))
    else:
        scope[tree.children[0].value] = value
