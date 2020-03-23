from ctypes import *

import llvmlite.binding as llvm


# All these initializations are required for code generation!
from volpe_types import int1, int32, flt32, flt64, VolpeTuple, VolpeClosure

llvm.initialize()
llvm.initialize_native_target()
llvm.initialize_native_asmprinter()  # yes, even this one


def compile_and_run(llvm_ir, result_type):
    """
    Create an ExecutionEngine suitable for JIT code generation on
    the host CPU.  The engine is reusable for an arbitrary number of
    modules.
    """
    # Create a target machine representing the host
    target = llvm.Target.from_default_triple()
    target_machine = target.create_target_machine()
    # And an execution engine with an empty backing module
    mod = llvm.parse_assembly(llvm_ir)
    mod.verify()
    engine = llvm.create_mcjit_compiler(mod, target_machine)
    engine.finalize_object()
    engine.run_static_constructors()

    func_ptr = engine.get_function_address("main")

    func = CFUNCTYPE(determine_c_type(result_type))(func_ptr)
    res = func()
    if hasattr(res, "contents"):
        print("main() =", res.contents)
    else:
        print("main() =", res)


def determine_c_type(volpe_type, depth=0):
    # Interpret result type.
    if isinstance(volpe_type, VolpeTuple):
        elems = volpe_type.elements
        class CTuple(Structure):
            _fields_ = [(f"elem{i}", determine_c_type(elem, depth+1)) for i, elem in enumerate(elems)]
            def __repr__(self):
                return "[" + ", ".join([str(getattr(self, tup[0])) for tup in self._fields_]) + "]"
        
        return POINTER(CTuple) if depth == 0 else CTuple
    if isinstance(volpe_type, VolpeClosure):
        class CFunc(Structure):
            _fields_ = [("func", POINTER(None)), ("c_func", POINTER(None)), ("f_func", POINTER(None)), ("env", POINTER(None))]
            def __repr__(self):
                return "*function*"
        return CFunc
    elif volpe_type == int1:
        return c_bool
    elif volpe_type == int32:
        return c_int32
    elif volpe_type == flt32:
        return c_float
    elif volpe_type == flt64:
        return c_double
    else:
        return None
