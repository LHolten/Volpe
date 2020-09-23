from copy import deepcopy
from dataclasses import dataclass
from typing import Dict, Union

from llvmlite import ir

from llvm_utils import options, build_func
from tree import TypeTree

int1 = ir.IntType(1)
int32 = ir.IntType(32)
int64 = ir.IntType(64)
int8 = ir.IntType(8)
pint8 = int8.as_pointer()
# flt32 = ir.FloatType()
flt64 = ir.DoubleType()
char = ir.IntType(8)
unknown = ir.VoidType()


class VolpeType:
    def __repr__(self):
        raise NotImplementedError()

    def unwrap(self) -> ir.Type:
        raise NotImplementedError()

    def __hash__(self):
        raise NotImplementedError()


def unwrap(value: Union[ir.Type, VolpeType]) -> ir.Type:
    if isinstance(value, VolpeType):
        return value.unwrap()
    return value


def is_int(value: Union[ir.Type, VolpeType]) -> bool:
    if isinstance(value, VolpeArray):
        return is_int(value.element)
    return value == int64


def is_flt(value: Union[ir.Type, VolpeType]) -> bool:
    if isinstance(value, VolpeArray):
        return is_flt(value.element)
    return value == flt64


def is_char(value: Union[ir.Type, VolpeType]) -> bool:
    if isinstance(value, VolpeArray):
        return is_char(value.element)
    return value == char


def int1_like(value: Union[ir.Type, VolpeType]) -> Union[ir.Type, VolpeType]:
    if isinstance(value, VolpeArray):
        return VolpeArray(int1_like(value.element), value.count)
    return int1


def is_pointer(value: Union[ir.Type, VolpeType]) -> bool:
    return (
        isinstance(value, VolpePointer) or
        isinstance(value, VolpeObject) and any(is_pointer(val) for val in value.type_dict.values()) or
        isinstance(value, VolpeArray) and is_pointer(value.element)
    )


@dataclass
class VolpeObject(VolpeType):
    type_dict: Dict[str, Union[ir.Type, VolpeType]]

    def __repr__(self):
        return "{" + ", ".join(str(v) for v in self.type_dict.values()) + "}"

    def unwrap(self) -> ir.Type:
        obj = ir.LiteralStructType(unwrap(value) for value in self.type_dict.values())
        obj.type_dict = self.type_dict
        return obj

    def __hash__(self):
        return hash(tuple(self.type_dict.values()))


@dataclass
class VolpeArray(VolpeType):
    element: Union[ir.Type, VolpeType]
    count: int

    def __repr__(self):
        return f"[{self.count} x {self.element}]"

    def unwrap(self) -> ir.Type:
        if self.count == 0:
            return ir.LiteralStructType([])
        return ir.VectorType(unwrap(self.element), self.count)

    def __hash__(self):
        return hash((self.element, self.count))


@dataclass
class VolpePointer(VolpeType):
    pointee: Union[ir.Type, VolpeType]

    def __repr__(self):
        return f"&{self.pointee}"

    def unwrap(self) -> ir.Type:
        return ir.PointerType(unwrap(self.pointee))

    def __hash__(self):
        return hash(self.pointee)


@dataclass
class VolpeClosure(VolpeType):
    tree: TypeTree
    scope: callable
    env: Dict[str, Union[ir.Type, VolpeType]] = None

    def __repr__(self):
        if self.env is None:
            return "{?}"
        return "{" + ", ".join(f"{k}: {v}" for k, v in self.env.items()) + "}"

    def unwrap(self) -> ir.Type:
        if self.env is None:
            self.env = dict()  # fix for passing around functions that are never called
        return ir.LiteralStructType(unwrap(value) for value in self.env.values())

    def __hash__(self):
        return hash(self.scope)

    def __eq__(self, other):
        return isinstance(other, VolpeClosure) and self.scope is other.scope

    def ret_type(self, parent, args: VolpeType):
        if args not in self.tree.instances:
            new_tree = self.tree.instances[args] = deepcopy(self.tree.children[0])
            self.tree.children.append(new_tree)

            self.env = dict()

            def scope(name, t_tree: TypeTree):
                if name == "@":
                    return self
                self.env[name] = self.scope(name, t_tree)
                return self.env[name]

            parent.__class__(new_tree.children[1], scope, (new_tree.children[0], args), stack_trace=parent.stack_trace)
        return self.tree.instances[args].children[1].return_type

    def build_or_get_function(self, parent, args):
        inst = self.tree.instances[args]
        if not hasattr(inst, "func"):
            arg_type = inst.children[0].return_type
            ret_type = inst.children[1].return_type

            module = parent.builder.module
            func_name = str(next(module.func_count))
            func_type = ir.FunctionType(unwrap(ret_type), [unwrap(self), unwrap(arg_type)])
            inst.func = ir.Function(module, func_type, func_name)

            with build_func(inst.func) as (b, args):
                b: ir.IRBuilder
                with options(b, args[1].type) as (rec, phi):
                    rec(args[1])

                new_values = [b.extract_value(args[0], i) for i in range(len(self.env))]
                env_scope = dict(zip(self.env.keys(), new_values))
                env_scope["@"] = args[0]

                def scope(name):
                    return env_scope[name]

                parent.__class__(b, inst.children[1], scope, b.ret, rec, (inst.children[0], phi))

        return inst.func
