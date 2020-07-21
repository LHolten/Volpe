from dataclasses import dataclass, field
from typing import Dict, Union

from llvmlite import ir
from unification import unifiable, isvar

from tree import TypeTree
from unification_copy import var

int1 = ir.IntType(1)
int32 = ir.IntType(32)
int64 = ir.IntType(64)
int8 = ir.IntType(8)
pint8 = int8.as_pointer()
# flt32 = ir.FloatType()
flt64 = ir.DoubleType()
char = ir.IntType(8)
unknown = ir.VoidType()


def vary():
    return field(default_factory=var)


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


@unifiable
@dataclass
class VolpeObject(VolpeType):
    type_dict: Dict[str, Union[ir.Type, VolpeType]] = vary()

    def __repr__(self):
        return "{" + ", ".join(str(v) for v in self.type_dict.values()) + "}"

    def unwrap(self) -> ir.Type:
        return ir.LiteralStructType(unwrap(value) for value in self.type_dict.values())

    def __hash__(self):
        return hash(tuple(self.type_dict.values()))


@unifiable
@dataclass
class VolpeArray(VolpeType):
    element: Union[ir.Type, VolpeType] = vary()
    count: int = vary()

    def __repr__(self):
        return f"[{self.count} x {self.element}]"

    def unwrap(self) -> ir.Type:
        return ir.VectorType(unwrap(self.element), self.count)

    def __hash__(self):
        return hash((self.element, self.count))


@unifiable
@dataclass
class VolpeClosure(VolpeType):
    tree: TypeTree = vary()
    scope: callable = vary()
    env: Dict[str, Union[ir.Type, VolpeType]] = vary()

    def __repr__(self):
        if isvar(self.env):
            return "{?}"
        return "{" + ", ".join(f"{k}: {v}" for k, v in self.env.items()) + "}"

    def unwrap(self) -> ir.Type:
        if isvar(self.env):
            self.env = dict()  # fix for passing around functions that are never called
        return ir.LiteralStructType(unwrap(value) for value in self.env.values())

    def __hash__(self):
        return hash(self.scope)

    def __eq__(self, other):
        return isinstance(other, VolpeClosure) and self.scope is other.scope
