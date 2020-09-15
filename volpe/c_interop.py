from dataclasses import dataclass

from llvmlite import ir
import pydffi as dffi

from llvm_utils import build_func
from volpe_types import int64, VolpeType, VolpeObject, VolpePointer, char, pint8, unknown, unwrap, int32


def volpe_from_c(c_type: dffi.Type):
    if isinstance(c_type, dffi.QualType):
        return volpe_from_c(c_type.type)
    if isinstance(c_type, dffi.BasicType):
        if c_type.kind == dffi.BasicKind.Int:
            return int64
        if c_type.size == 1:
            return char
        assert False, f"unknown basic-type: {c_type.kind}"
    if isinstance(c_type, dffi.PointerType):
        return VolpePointer(volpe_from_c(c_type.pointee()))
    assert False, f"unknown c-type: {c_type}"


@dataclass
class VolpeCFunc(VolpeType):
    c_func: dffi.FunctionType
    name: str

    def __repr__(self):
        return "c-func"

    def unwrap_args(self):
        return [unwrap(volpe_from_c(p)) for p in self.c_func.params]

    def unwrap_ret(self):
        return unwrap(volpe_from_c(self.c_func.returnType))

    def unwrap(self) -> ir.Type:
        args = ir.LiteralStructType(self.unwrap_args())
        if len(args) == 1:
            args = args.elements[0]
        return ir.FunctionType(self.unwrap_ret(), [ir.LiteralStructType([]), args])

    def __hash__(self):
        raise hash(self.c_func)

    def ret_type(self, parent, args: VolpeType):
        if not isinstance(args, VolpeObject):
            args = VolpeObject({"_0": args})
        target_args = VolpeObject({f"_{i}": volpe_from_c(p) for i, p in enumerate(self.c_func.params)})
        assert args == target_args
        return volpe_from_c(self.c_func.returnType)

    def build_or_get_function(self, parent, args):
        module: ir.Module = parent.builder.module
        func_type = ir.FunctionType(self.unwrap_ret(), self.unwrap_args())
        func = ir.Function(module, func_type, self.name)

        wrapper_type = ir.FunctionType(unknown,
                                       [func_type.as_pointer(), func_type.return_type.as_pointer(), pint8.as_pointer()])
        wrapper_name = f"{self.name}_wrapper"
        wrapper = ir.Function(module, wrapper_type, wrapper_name)

        volpe_func_type = self.unwrap()
        volpe_func = ir.Function(module, volpe_func_type, str(next(module.func_count)))
        with build_func(volpe_func) as (b, args):
            b: ir.IRBuilder
            args = args[1]
            if not isinstance(args.type, ir.LiteralStructType):
                new_args = ir.LiteralStructType([args.type])(ir.Undefined)
                args = b.insert_value(new_args, args, 0)

            ptr = b.alloca(args.type)
            b.store(args, ptr)
            params = ir.LiteralStructType([ir.PointerType(t) for t in args.type.elements])(ir.Undefined)

            for i in range(len(args.type.elements)):
                params = b.insert_value(params, b.gep(ptr, [int32(0), int32(i)]), i)

            param_ptr = b.alloca(params.type)
            b.store(params, param_ptr)
            param_ptr = b.bitcast(param_ptr, pint8.as_pointer())

            result_ptr = b.alloca(self.unwrap_ret())

            b.call(wrapper, [func, result_ptr, param_ptr])

            b.ret(b.load(result_ptr))

        module.wrappers[wrapper_name] = self.c_func.getWrapperLLVMStr(wrapper_name)

        return volpe_func
