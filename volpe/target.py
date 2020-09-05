import llvmlite.binding as llvm

# All these initializations are required for code generation!
llvm.initialize()
llvm.initialize_native_target()
llvm.initialize_native_asmprinter()  # yes, even this one

# Could be useful if you want to compile for other targets.
# llvmlite.binding.initialize_all_targets()

# Ensure JIT execution is allowed
llvm.check_jit_execution()

target = llvm.Target.from_triple(llvm.get_process_triple())
target_machine = target.create_target_machine(codemodel="default")
target_data = target_machine.target_data

# Configure optimization pass manager builder
# https://llvmlite.readthedocs.io/en/latest/user-guide/binding/optimization-passes.html#llvmlite.binding.PassManagerBuilder
pm_builder = llvm.PassManagerBuilder()
pm_builder.disable_unroll_loops = False
pm_builder.inlining_threshold = 100
pm_builder.loop_vectorize = True
pm_builder.slp_vectorize = True
pm_builder.opt_level = 3
pm_builder.size_level = 0

pass_manager = llvm.ModulePassManager()
pm_builder.populate(pass_manager)

# Target specific optimizations
target_machine.add_analysis_passes(pass_manager)