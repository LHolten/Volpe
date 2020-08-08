import io
import time
from ctypes import CFUNCTYPE, POINTER, byref

import llvmlite.binding as llvm

from volpe_repr import determine_c_type, ENCODING

# All these initializations are required for code generation!
llvm.initialize()
llvm.initialize_native_target()
llvm.initialize_native_asmprinter()  # yes, even this one

# Could be useful if you want to compile for other targets.
# llvmlite.binding.initialize_all_targets() 

# Ensure JIT execution is allowed
llvm.check_jit_execution()


def compile_and_run(llvm_ir, result_type, more_verbose=False, show_time=False, console=False):
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

    # Optimizations

    # https://llvmlite.readthedocs.io/en/latest/user-guide/binding/optimization-passes.html#llvmlite.binding.PassManagerBuilder
    pm_builder = llvm.PassManagerBuilder()
    pm_builder.disable_unroll_loops = False
    pm_builder.inlining_threshold = 1000
    pm_builder.loop_vectorize = True
    pm_builder.slp_vectorize = True
    pm_builder.opt_level = 3
    pm_builder.size_level = 0

    pm = llvm.ModulePassManager()
    pm_builder.populate(pm)

    # target specific optimizations
    target_machine.add_analysis_passes(pm)
    
    if more_verbose:
        print("\nBefore optimization\n")
        print(mod)

    pm.run(mod)

    if more_verbose:
        print("\nAfter optimization\n")
        print(mod)

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

    if hasattr(res, "value"):
        res = res.value
    if hasattr(res, "decode"):
        res = res.decode(ENCODING)
    print("main() =", repr(res))
    
    if show_time:
        time_taken = end_time - start_time
        if time_taken > 100000:
            print(f"time = {time_taken/1E9}s")
        else:
            print(f"time = {time_taken}ns")
