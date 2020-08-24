import io
from ctypes import CFUNCTYPE, POINTER, byref

import llvmlite.binding as llvm

from volpe_repr import determine_c_type, get_repr

import time
from sys import version_info

# use nanosecond perf_counter if available
if version_info >= (3, 7, 0):
    perf_counter = time.perf_counter_ns
    ticks_in_sec = 1e9
else:
    perf_counter = time.perf_counter
    ticks_in_sec = 1

MEASUREMENT_TIME = 5.0 * ticks_in_sec

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

    # Configure optimization pass manager builder
    # https://llvmlite.readthedocs.io/en/latest/user-guide/binding/optimization-passes.html#llvmlite.binding.PassManagerBuilder
    pm_builder = llvm.PassManagerBuilder()
    pm_builder.disable_unroll_loops = False
    pm_builder.inlining_threshold = 100
    pm_builder.loop_vectorize = True
    pm_builder.slp_vectorize = True
    pm_builder.opt_level = 3
    pm_builder.size_level = 0

    pm = llvm.ModulePassManager()
    pm_builder.populate(pm)

    # Target specific optimizations
    target_machine.add_analysis_passes(pm)

    if more_verbose:
        # print("\nBefore optimization\n")
        # print(mod)
        with open("before.ll", "w") as file:
            file.write(str(mod))

    # Run the optimization passes
    pm.run(mod)

    if more_verbose:
        # print("\nAfter optimization\n")
        # print(mod)
        with open("after.ll", "w") as file:
            file.write(str(mod))

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

    if show_time:
        count = 0
        start_time = perf_counter()
        while perf_counter() - start_time < MEASUREMENT_TIME:
            func(byref(res))
            count += 1
        end_time = perf_counter()
    else:
        func(byref(res))

    print("main() =", get_repr(res))

    if show_time:
        if count > 1:
            print(f"ran the code {count} times")
            nanoseconds = (end_time - start_time) / count * (1e9 / ticks_in_sec)
            if nanoseconds > 1e9:
                print(f"average time: {nanoseconds / 1E9:.3f} s")
            elif nanoseconds > 1e6:
                print(f"average time: {nanoseconds / 1E6:.3f} ms")
            elif nanoseconds > 1e3:
                print(f"average time: {nanoseconds / 1E3:.3f} µs")
            else:
                print(f"average time: {nanoseconds:.3f} ns")
        else:
            print("ran the code once")
            print(f"time: {(end_time - start_time) / ticks_in_sec:.3f} s")
