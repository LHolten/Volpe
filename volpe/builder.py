from typing import Callable

from lark.visitors import Interpreter
from llvmlite import ir

from builder_utils import write_environment, free_environment, options, \
    read_environment, tuple_assign, build_func, copy, copy_environment
from tree import TypeTree
from volpe_types import int1, flt32, flt64, copy_func, free_func, Closure


class LLVMScope(Interpreter):
    def __init__(self, builder: ir.IRBuilder, tree: TypeTree, scope: dict, ret: Callable, fast=False):
        self.builder = builder
        self.scope = scope
        self.ret = ret
        self.fast = fast

        if tree.data == "block":
            assert len(tree.children) > 0, "code block needs code"

            def evaluate(children):
                if len(children) == 1:
                    self.visit_unsafe(children[0])
                else:
                    success = self.visit(children[0])
                    with self.builder.if_then(success):
                        evaluate(children[1:])
                    builder.unreachable()

            evaluate(tree.children)
            assert builder.block.is_terminated, "you forgot a return statement at the end of a code block"
        else:
            value = self.visit(tree)
            free_environment(builder, scope.values())
            ret(value)

    def visit(self, tree: TypeTree):
        value = getattr(self, tree.data)(tree)
        assert not self.builder.block.is_terminated, "dead code is not allowed"
        return value

    def visit_unsafe(self, tree: TypeTree):
        return getattr(self, tree.data)(tree)

    def assign(self, tree: TypeTree):
        tuple_assign(self.builder, self.scope, tree.children[0], self.visit(tree.children[1]))
        return int1(True)

    def symbol(self, tree: TypeTree):
        return copy(self.builder, self.scope[tree.children[0].value])

    def func(self, tree: TypeTree):
        func_type = tree.return_type
        assert isinstance(func_type, Closure)

        module = self.builder.module
        env_names = list(func_type.outside_used)
        env_values = [self.scope[name] for name in env_names]
        env_types = [value.type for value in env_values]

        func_name = str(next(module.func_count))
        func = ir.Function(module, func_type.func, func_name)
        c_func = ir.Function(module, copy_func, func_name + ".copy")
        f_func = ir.Function(module, free_func, func_name + ".free")
        closure = func_type([func, c_func, f_func, ir.Undefined])

        with build_func(func) as (b, args):
            if func_type.checked:
                new_values = copy_environment(b, read_environment(b, args[0], env_types))

                new_scope = dict(zip(env_names, new_values))
                for shape, value in zip(tree.children[:-1], args[1:]):
                    tuple_assign(b, new_scope, shape, value)
                new_scope["@"] = b.insert_value(closure, b.call(c_func, [args[0]]), 3)

                LLVMScope(b, tree.children[-1], new_scope, b.ret, fast=self.fast)
            else:
                b.ret_void()
                print("ignoring function without usage")

        with build_func(c_func) as (b, args):
            b.ret(write_environment(b, copy_environment(b, read_environment(b, args[0], env_types))))

        with build_func(f_func) as (b, args):
            free_environment(b, read_environment(b, args[0], env_types))
            b.call(b.module.free, args)
            b.ret_void()

        env_ptr = write_environment(self.builder, copy_environment(self.builder, env_values))
        return self.builder.insert_value(closure, env_ptr, 3)

    def func_call(self, tree: TypeTree):
        values = self.visit_children(tree)
        closure = values[0]
        args = values[1:]

        assert isinstance(closure.type, Closure)

        func = self.builder.extract_value(closure, 0)
        f_func = self.builder.extract_value(closure, 2)
        env_ptr = self.builder.extract_value(closure, 3)

        value = self.builder.call(func, [env_ptr, *args])
        self.builder.call(f_func, [env_ptr])
        return value

    def this_func(self, tree: TypeTree):
        return copy(self.builder, self.scope["@"])

    def returnn(self, tree: TypeTree):
        value = self.visit(tree.children[0])
        free_environment(self.builder, self.scope.values())
        self.ret(value)

    def block(self, tree: TypeTree):
        phi = []

        new_scope = dict(zip(self.scope.keys(), copy_environment(self.builder, self.scope.values())))

        with options(self.builder, tree.return_type, phi) as ret:
            LLVMScope(self.builder, tree, new_scope, ret, fast=self.fast)

        return phi[0]

    def implication(self, tree: TypeTree):
        phi = []

        with options(self.builder, tree.return_type, phi) as ret:
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(self.visit_unsafe(tree.children[1]))
            ret(int1(1))

        return phi[0]

    def logic_and(self, tree: TypeTree):
        phi = []

        with options(self.builder, tree.return_type, phi) as ret:
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(self.visit_unsafe(tree.children[1]))
            ret(int1(0))

        return phi[0]

    def logic_or(self, tree: TypeTree):
        phi = []

        with options(self.builder, tree.return_type, phi) as ret:
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(int1(1))
            ret(self.visit_unsafe(tree.children[1]))

        return phi[0]

    def logic_not(self, tree: TypeTree):
        value = self.visit_children(tree)[0]
        return self.builder.not_(value)

    def collect_tuple(self, tree: TypeTree):
        value = tree.return_type(ir.Undefined)
        for i, v in enumerate(self.visit_children(tree)):
            value = self.builder.insert_value(value, v, i)
        return value
        
    # Integers
    def integer(self, tree: TypeTree):
        return tree.return_type(tree.children[0].value)

    def add_int(self, tree: TypeTree):
        # TODO Use overflow bit to raise runtime error
        # self.builder.extract_value(self.builder.sadd_with_overflow(values[0], values[1]), 0)
        values = self.visit_children(tree)
        return self.builder.add(values[0], values[1])

    def sub_int(self, tree: TypeTree):
        # TODO Use overflow bit to raise runtime error
        # self.builder.extract_value(self.builder.ssub_with_overflow(values[0], values[1]), 0)
        values = self.visit_children(tree)
        return self.builder.sub(values[0], values[1])

    def mod_int(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.srem(values[0], values[1])

    def div_int(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.sdiv(values[0], values[1])

    def mul_int(self, tree: TypeTree):
        # TODO Use overflow bit to raise runtime error
        # self.builder.extract_value(self.builder.smul_with_overflow(values[0], values[1]), 0)
        values = self.visit_children(tree)
        return self.builder.mul(values[0], values[1])

    def negate_int(self, tree: TypeTree):
        value = self.visit_children(tree)[0]
        return self.builder.neg(value)

    def convert_int(self, tree: TypeTree):
        value = self.visit_children(tree)[0]
        if self.fast:
            return self.builder.sitofp(value, flt32)
        return self.builder.sitofp(value, flt64)

    def equals_int(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed("==", values[0], values[1])

    def not_equals_int(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed("!=", values[0], values[1])

    def greater_int(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed(">", values[0], values[1])

    def less_int(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed("<", values[0], values[1])

    def greater_equals_int(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed(">=", values[0], values[1])

    def less_equals_int(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed("<=", values[0], values[1])

    # Floating point numbers
    def floating(self, tree: TypeTree):
        return tree.return_type(float(tree.children[0].value))

    def add_flt(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.fadd(values[0], values[1])

    def sub_flt(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.fsub(values[0], values[1])

    def mod_flt(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.frem(values[0], values[1])

    def div_flt(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.fdiv(values[0], values[1])

    def mul_flt(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.fmul(values[0], values[1])

    def negate_flt(self, tree: TypeTree):
        value = self.visit_children(tree)[0]
        if self.fast:
            return self.builder.fsub(flt32(0), value)
        return self.builder.fsub(flt64(0), value)

    # FLOAT TO INT DISABLED
    # def convert_flt(self, tree: TypeTree):
    #     value = self.visit_children(tree)[0]
    #     return self.builder.fptosi(value, int32)

    def equals_flt(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.fcmp_ordered("==", values[0], values[1])

    def not_equals_flt(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.fcmp_ordered("!=", values[0], values[1])

    def greater_flt(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.fcmp_ordered(">", values[0], values[1])

    def less_flt(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.fcmp_ordered("<", values[0], values[1])

    def greater_equals_flt(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.fcmp_ordered(">=", values[0], values[1])

    def less_equals_flt(self, tree: TypeTree):
        values = self.visit_children(tree)
        return self.builder.fcmp_ordered("<=", values[0], values[1])

    def __default__(self, tree: TypeTree):
        raise NotImplementedError("llvm", tree.data)
