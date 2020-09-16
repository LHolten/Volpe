import io
from ctypes import CFUNCTYPE, POINTER, byref

from target import target_machine, pass_manager, llvm
from volpe_repr import determine_c_type, get_repr
from version_dependent import perf_counter, TICKS_IN_SEC

MEASUREMENT_TIME = 5.0 * TICKS_IN_SEC


def compile_and_run(llvm_ir, result_type, more_verbose=False, show_time=False, console=False):
    """
    Create an ExecutionEngine suitable for JIT code generation on
    the host CPU. The engine is reusable for an arbitrary number of
    modules.
    """
    if more_verbose:
        # print("\nBefore optimization\n")
        # print(mod)
        with open("before.ll", "w") as file:
            file.write(llvm_ir)

    # And an execution engine with an empty backing module
    mod = llvm.parse_assembly(llvm_ir)
    mod.triple = llvm.get_process_triple()
    mod.verify()

    # Run the optimization passes
    pass_manager.run(mod)

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

    print(get_repr(res))

    if show_time:
        if count > 1:
            print(f"ran the code {count} times")
            nanoseconds = (end_time - start_time) / count * (1e9 / TICKS_IN_SEC)
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
            print(f"time: {(end_time - start_time) / TICKS_IN_SEC:.3f} s")
