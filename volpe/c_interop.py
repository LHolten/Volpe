from dataclasses import dataclass
from typing import Union

from llvmlite import ir
import clang.cindex

from llvm_utils import build_func
from volpe_types import int64, VolpeType, VolpeObject, VolpePointer, char, pint8, unknown, unwrap, int32

index = clang.cindex.Index.create()


def volpe_from_c(c_type: clang.cindex.Type) -> Union[ir.Type, VolpeType]:
    if c_type.kind == clang.cindex.TypeKind.POINTER:
        return VolpePointer(volpe_from_c(c_type.get_pointee()))
    if c_type.kind == clang.cindex.TypeKind.CHAR_S:
        return char
    if c_type.kind == clang.cindex.TypeKind.INT:
        return int64

    assert False, f"unknown c-type: {c_type.kind}"


@dataclass
class VolpeCFunc(VolpeType):
    c_func: clang.cindex.Type
    name: str

    def __repr__(self):
        return "c-func"

    def args(self):
        args = [volpe_from_c(a) for a in self.c_func.argument_types()]
        if len(args) == 1:
            return args[0]
        return VolpeObject({f"_{i}": a for i, a in enumerate(args)})

    def ret(self):
        return volpe_from_c(self.c_func.get_result())

    def unwrap(self) -> ir.Type:
        return ir.LiteralStructType([])

    def __hash__(self):
        raise hash(self.c_func)

    def ret_type(self, parent, args: VolpeType):
        assert args == self.args()
        return self.ret()

    def build_or_get_function(self, parent, volpe_args):
        module: ir.Module = parent.builder.module

        func_args = unwrap(self.args())
        if not isinstance(volpe_args, VolpeObject):
            func_args = ir.LiteralStructType([func_args])
        func_type = ir.FunctionType(unwrap(self.ret()), func_args)
        func = ir.Function(module, func_type, self.name)

        volpe_func_type = ir.FunctionType(unwrap(self.ret()), [ir.LiteralStructType([]), unwrap(self.args())])
        volpe_func = ir.Function(module, volpe_func_type, str(next(module.func_count)))
        with build_func(volpe_func) as (b, args):
            b: ir.IRBuilder
            args = [args[1]]
            if isinstance(volpe_args, VolpeObject):
                args = [b.extract_value(args[0], i) for i in range(len(volpe_args.type_dict))]

            b.ret(b.call(func, args))

        return volpe_func
