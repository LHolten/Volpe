from ctypes import *

import llvmlite.binding as llvm


# All these initializations are required for code generation!
from volpe_types import (
    int1, 
    int32, 
    flt32, 
    flt64, 
    VolpeTuple, 
    VolpeClosure,
    VolpeIterator,
    VolpeList
)

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
    """Interpret the volpe type and return a corresponding C type."""
    # Simple types:
    if volpe_type == int1:
        return c_bool
    elif volpe_type == int32:
        return c_int32
    elif volpe_type == flt32:
        return c_float
    elif volpe_type == flt64:
        return c_double
    
    # Aggregate types:
    elif isinstance(volpe_type, VolpeTuple):
        print(volpe_type._to_string())
        elems = volpe_type.elements
        class CTuple(Structure):
            _fields_ = [(f"elem{i}", determine_c_type(elem, depth+1)) for i, elem in enumerate(elems)]
            def __repr__(self):
                return "[" + ", ".join([str(getattr(self, tup[0])) for tup in self._fields_]) + "]"
        return POINTER(CTuple) if depth == 0 else CTuple

    elif isinstance(volpe_type, VolpeClosure):
        print(volpe_type._to_string())
        class CFunc(Structure):
            _fields_ = [("func", POINTER(None)), ("c_func", POINTER(None)), ("f_func", POINTER(None)), ("env", POINTER(None))]
            def __repr__(self):
                return "*function*"
        return CFunc

    elif isinstance(volpe_type, VolpeIterator):
        print(volpe_type._to_string())
        class CFunc(Structure):
            _fields_ = [("func", POINTER(None)), ("c_func", POINTER(None)), ("f_func", POINTER(None)), ("env", POINTER(None))]
        class CIterator(Structure):
            _fields_ = [("func", CFunc), ("num", c_int32)]
            def __repr__(self):
                return "*iterator*"
        return CIterator

    elif isinstance(volpe_type, VolpeList):
        print(volpe_type._to_string())
        class CList(Structure):
            _fields_ = [("elem_type", POINTER(None)), ("num", c_int32)]
            def __repr__(self):
                return "*list*"
        return CList
        
    # Unknown type
    else: return None
