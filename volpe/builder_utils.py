from contextlib import contextmanager
from typing import List, Dict, Iterable

from llvmlite import ir

from tree import TypeTree
from volpe_types import int32, target_data, VolpeTuple, Closure, copy_func, free_func, VolpeList


def free(b, value):
    if isinstance(value.type, Closure):
        f_func = b.extract_value(value, 2)
        env_ptr = b.extract_value(value, 3)
        b.call(f_func, [env_ptr])
    if isinstance(value.type, VolpeTuple):
        for i in range(len(value.type.elements)):
            free(b, b.extract_value(value, i))
    if isinstance(value.type, VolpeList):
        free(b, b.extract_value(value, 0))


def copy(b, value):
    if isinstance(value.type, Closure):
        env_copy = b.call(b.extract_value(value, 1), [b.extract_value(value, 3)])
        value = b.insert_value(value, env_copy, 3)
    if isinstance(value.type, VolpeTuple):
        for i in range(len(value.type.elements)):
            value = b.insert_value(value, copy(b, b.extract_value(value, i)), i)
    if isinstance(value.type, VolpeList):
        closure = b.extract_value(value, 0)
        value = b.insert_value(value, copy(b, closure), 0)
    return value


def write_environment(b: ir.IRBuilder, value_list: List):
    env_type = ir.LiteralStructType([value.type for value in value_list])
    untyped_ptr = b.call(b.module.malloc, [int32(env_type.get_abi_size(target_data))])
    ptr = b.bitcast(untyped_ptr, env_type.as_pointer())

    for i, value in enumerate(value_list):
        b.store(value, b.gep(ptr, [int32(0), int32(i)]))

    return untyped_ptr


def read_environment(b: ir.IRBuilder, untyped_ptr: ir.NamedValue, type_list: List) -> List[ir.NamedValue]:
    env_type = ir.LiteralStructType(type_list)
    ptr = b.bitcast(untyped_ptr, env_type.as_pointer())

    value_list = []
    for i, t in enumerate(type_list):
        value = b.load(b.gep(ptr, [int32(0), int32(i)]))
        value_list.append(value)

    return value_list


def copy_environment(b: ir.IRBuilder, value_list: Iterable) -> list:
    return [copy(b, value) for value in value_list]


def free_environment(b: ir.IRBuilder, value_list: Iterable) -> None:
    [free(b, value) for value in value_list]


@contextmanager
def options(b: ir.IRBuilder, t: ir.Type) -> ir.Value:
    new_block = b.function.append_basic_block("block")
    with b.goto_block(new_block):
        phi_node = b.phi(t)

    def ret(value):
        if not b.block.is_terminated:
            phi_node.add_incoming(value, b.block)
            b.branch(new_block)

    yield ret, phi_node

    b.position_at_end(new_block)


@contextmanager
def build_func(func: ir.Function):
    block = func.append_basic_block("entry")
    builder = ir.IRBuilder(block)

    yield builder, func.args


@contextmanager
def build_closure(module, closure_type, env_types):
    func_name = str(next(module.func_count))
    func = ir.Function(module, closure_type.func, func_name)
    c_func = ir.Function(module, copy_func, func_name + ".copy")
    f_func = ir.Function(module, free_func, func_name + ".free")
    closure = closure_type([func, c_func, f_func, ir.Undefined])

    with build_func(func) as (b, args):
        yield b, args, closure, c_func

    with build_func(c_func) as (b, args):
        b.ret(write_environment(b, copy_environment(b, read_environment(b, args[0], env_types))))

    with build_func(f_func) as (b, args):
        free_environment(b, read_environment(b, args[0], env_types))
        b.call(b.module.free, args)
        b.ret_void()


def closure_call(b, closure, args):
    func = b.extract_value(closure, 0)
    env_ptr = b.extract_value(closure, 3)

    value = b.call(func, [env_ptr, *args])
    return value


def tuple_assign(b: ir.IRBuilder, scope: Dict, shape: TypeTree, value):
    if shape.data == "shape":
        for i, child in enumerate(shape.children):
            tuple_assign(b, scope, child, b.extract_value(value, i))
    else:
        name = shape.children[0].value
        if name in scope:
            free(b, scope[name])
        scope[name] = value
