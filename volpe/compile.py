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
    if volpe_type == int32:
        return c_int32
    if volpe_type == flt32:
        return c_float
    if volpe_type == flt64:
        return c_double
    
    # Aggregate types:
    if isinstance(volpe_type, VolpeTuple):
        elems = volpe_type.elements
        class CTuple(Structure):
            _fields_ = [(f"elem{i}", determine_c_type(elem, depth+1)) for i, elem in enumerate(elems)]
            def __repr__(self):
                return "[" + ", ".join([str(getattr(self, tup[0])) for tup in self._fields_]) + "]"
        return POINTER(CTuple) if depth == 0 else CTuple

    if isinstance(volpe_type, VolpeList):
        element_type = determine_c_type(volpe_type.element_type, depth+1)
        class CList(Structure):
            _fields_ = [("elems", POINTER(element_type)), ("length", c_int32)]
            def __repr__(self):
                if depth == 0:
                    elems = getattr(self, "elems")
                    length = getattr(self, "length")
                    return "&<" + ", ".join([str(elem) for elem in elems[:length]]) + ">"
                return get_type_name(volpe_type)
        return POINTER(CList) if depth == 0 else CList

    if isinstance(volpe_type, VolpeClosure):
        elems = volpe_type.elements
        class CFunc(Structure):
            _fields_ = [("func", POINTER(None)), ("c_func", POINTER(None)), ("f_func", POINTER(None)), ("env", POINTER(None))]
            def __repr__(self):
                if depth == 0:
                    func = elems[0].pointee
                    input_type = ", ".join([get_type_name(i) for i in func.args[1:]])
                    return_type = get_type_name(func.return_type)
                    return f"function ({input_type}) => {return_type}"
                return get_type_name(volpe_type)
        return CFunc

    if isinstance(volpe_type, VolpeIterator):
        class CFunc(Structure):
            _fields_ = [("func", POINTER(None)), ("c_func", POINTER(None)), ("f_func", POINTER(None)), ("env", POINTER(None))]
        class CIterator(Structure):
            _fields_ = [("func", CFunc), ("length", c_int32)]
            def __repr__(self):
                if depth == 0:
                    # TODO maybe include length?
                    func = volpe_type.elements[0].elements[0].pointee
                    return_type = get_type_name(func.return_type)
                    return f"iterator of {return_type}"
                return get_type_name(volpe_type)
        return CIterator
        
    # Unknown type
    return None


def get_type_name(volpe_type):
    """Get short Volpe names for types."""
    # Simple types:
    if volpe_type == int1:
        return "bool"
    if volpe_type == int32:
        return "int32"
    if volpe_type == flt32:
        return "flt32"
    if volpe_type == flt64:
        return "flt64"

    # Aggregate types:
    if isinstance(volpe_type, VolpeTuple):
        type_reprs = ", ".join([get_type_name(elem) for elem in volpe_type.elements])
        return f"[{type_reprs}]"
    if isinstance(volpe_type, VolpeList):
        return "*list*"
    if isinstance(volpe_type, VolpeClosure):
        return "*function*"
    if isinstance(volpe_type, VolpeIterator):
        return "*iterator*"
        
    # Unknown type
    return None
