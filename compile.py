from ctypes import CFUNCTYPE, c_int32, c_bool, c_float

import llvmlite.binding as llvm


# All these initializations are required for code generation!
from util import int32, flt32

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
    llvm.load_library_permanently("msvcrt.dll")
    mod = llvm.parse_assembly(llvm_ir)
    mod.verify()
    engine = llvm.create_mcjit_compiler(mod, target_machine)
    engine.finalize_object()
    engine.run_static_constructors()

    func_ptr = engine.get_function_address("main")
    if result_type == int32:
        func = CFUNCTYPE(c_int32)(func_ptr)
    elif result_type == flt32:
        func = CFUNCTYPE(c_float)(func_ptr)
    else:
        func = CFUNCTYPE(c_bool)(func_ptr)

    res = func()
    print("main() =", res)
