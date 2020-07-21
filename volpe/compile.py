import io
import time
from ctypes import CFUNCTYPE, POINTER, byref

import llvmlite.binding as llvm

from volpe_repr import determine_c_type
from volpe_types import char, VolpeArray

# All these initializations are required for code generation!
llvm.initialize()
llvm.initialize_native_target()
llvm.initialize_native_asmprinter()  # yes, even this one


def compile_and_run(llvm_ir, result_type, show_time=False, console=False):
    """
    Create an ExecutionEngine suitable for JIT code generation on
    the host CPU. The engine is reusable for an arbitrary number of
    modules.
    """
    # Create a target machine representing the host
    target = llvm.Target.from_triple(llvm.get_process_triple())
    target_machine = target.create_target_machine(codemodel="default")
    # And an execution engine with an empty backing module
    mod = llvm.parse_assembly(llvm_ir)
    mod.triple = llvm.get_process_triple()
    mod.verify()

    if console:
        with io.open("output.obj", "wb") as file:
            file.write(target_machine.emit_object(mod))

    engine = llvm.create_mcjit_compiler(mod, target_machine)
    engine.finalize_object()
    engine.run_static_constructors()

    func_ptr = engine.get_function_address("run")

    c_type = determine_c_type(result_type)
    func = CFUNCTYPE(None, POINTER(c_type))(func_ptr)
    res = c_type()

    start_time = time.perf_counter_ns()
    func(byref(res))
    end_time = time.perf_counter_ns()

    # Properly decode strings and chars.
    if result_type == char:
        res = bytes(res).decode("ascii")
        res = f"\'{res}\'"
    elif isinstance(result_type, VolpeArray) and result_type.element == char:
        res = bytes(res).decode("ascii")
        res = f"\"{res}\""

    print("main() =", res)
    
    if show_time:
        time_taken = end_time - start_time
        if time_taken > 100000:
            print(f"time = {time_taken/1E9}s")
        else:
            print(f"time = {time_taken}ns")
