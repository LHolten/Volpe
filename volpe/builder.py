from typing import Callable

from lark.visitors import Interpreter
from llvmlite import ir

from volpe.annotate_utils import Unannotated
from volpe.builder_utils import write_environment, Closure, free_environment, options, \
    read_environment, tuple_assign, build_func, copy
from volpe.tree import TypeTree
from volpe.volpe_types import int1, make_bool, make_flt, flt32, copy_func, free_func


class LLVMScope(Interpreter):
    def __init__(self, builder: ir.IRBuilder, scope: dict, tree: TypeTree, ret: Callable, old_scope: set, closure):
        self.builder = builder
        self.scope = scope
        self.old_scope = old_scope
        self.ret = ret
        self.closure = closure

        if tree.data == "code":
            assert len(tree.children) > 0, "code block needs code"

            def evaluate(children):
                if len(children) == 1:
                    self.visit_unsafe(children[0])
                else:
                    value = self.visit(children[0])
                    with self.builder.if_then(value):
                        evaluate(children[1:])
                    builder.unreachable()

            evaluate(tree.children)
            assert builder.block.is_terminated, "you forgot a return statement at the end of a code block"
        else:
            value = self.visit(tree)

            scope = set(self.scope.values()) - self.old_scope
            free_environment(self.builder, scope)

            ret(value)

    def visit(self, tree: TypeTree):
        value = getattr(self, tree.data)(tree)
        assert not self.builder.block.is_terminated, "dead code is not allowed"
        return value

    def visit_unsafe(self, tree: TypeTree):
        return getattr(self, tree.data)(tree)

    def assign(self, tree: TypeTree):
        value = self.visit(tree.children[1])
        if value in self.scope.values():
            value = copy(self.builder, value)
        tuple_assign(self.scope, self.builder, tree.children[0], value)
        return ir.Constant(int1, True)

    def symbol(self, tree: TypeTree):
        return self.scope[tree.children[0].value]

    def func(self, tree: TypeTree):
        f = tree.ret
        assert isinstance(f, Unannotated)

        module = self.builder.module
        env_values = list(self.scope.values())
        env_types = list(v.type for v in env_values)
        env_names = list(self.scope.keys())

        func_name = str(next(module.func_count))

        c_func = ir.Function(module, copy_func, func_name + ".copy")
        with build_func(c_func) as (b, args):
            new_values = read_environment(b, args[0], env_types)
            new_ptr = write_environment(b, new_values)
            b.ret(new_ptr)

        f_func = ir.Function(module, free_func, func_name + ".free")
        with build_func(f_func) as (b, args):
            new_values = read_environment(b, args[0], env_types)
            free_environment(b, set(new_values))
            b.call(b.module.free, args)
            b.ret_void()

        func = ir.Function(module, f.func, func_name)
        with build_func(func) as (b, args):
            if f.checked:
                new_values = read_environment(b, args[0], env_types)
                new_scope = dict(zip(env_names, new_values))
                for a, t in zip(tree.children[:-1], args[1:]):
                    tuple_assign(new_scope, b, a, t)

                closure = ir.Constant(f, [func, c_func, f_func, ir.Undefined])
                closure = b.insert_value(closure, args[0], 3)

                LLVMScope(b, new_scope, tree.children[-1], b.ret, set(new_values), closure)
            else:
                b.ret_void()
                print("ignoring function without usage")

        closure = ir.Constant(f, [func, c_func, f_func, ir.Undefined])
        env_ptr = write_environment(self.builder, env_values)
        closure = self.builder.insert_value(closure, env_ptr, 3)
        return closure

    def func_call(self, tree: TypeTree):
        closure = self.visit(tree.children[0])
        args = [self.visit(child) for child in tree.children[1:]]
        for i, a in enumerate(args):
            if a in self.scope.values():
                args[i] = copy(self.builder, a)

        assert isinstance(closure.type, Closure)

        func = self.builder.extract_value(closure, 0)
        env_ptr = self.builder.extract_value(closure, 3)

        return self.builder.call(func, [env_ptr, *args])

    def this_func(self, tree: TypeTree):
        return self.closure

    def returnn(self, tree: TypeTree):
        value = self.visit(tree.children[0])

        scope = set(self.scope.values()) - {value} - self.old_scope
        free_environment(self.builder, scope)

        self.ret(value)

    def code(self, tree: TypeTree):
        phi = []

        with options(self.builder, tree.ret, phi) as ret:
            LLVMScope(self.builder, self.scope.copy(), tree, ret, set(self.scope.values()), self.closure)

        return phi[0]

    def implication(self, tree: TypeTree):
        phi = []

        with options(self.builder, tree.ret, phi) as ret:
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(self.visit_unsafe(tree.children[1]))
            ret(make_bool(1))

        return phi[0]

    def logic_and(self, tree: TypeTree):
        phi = []

        with options(self.builder, tree.ret, phi) as ret:
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(self.visit_unsafe(tree.children[1]))
            ret(make_bool(0))

        return phi[0]

    def logic_or(self, tree: TypeTree):
        phi = []

        with options(self.builder, tree.ret, phi) as ret:
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(make_bool(1))
            ret(self.visit_unsafe(tree.children[1]))

        return phi[0]

    def logic_not(self, tree: TypeTree):
        value = self.visit_children(tree)[0]
        return self.builder.not_(value)

    def collect_tuple(self, tree: TypeTree):
        value = ir.Constant(tree.ret, ir.Undefined)
        for i, v in enumerate(self.visit_children(tree)):
            value = self.builder.insert_value(value, v, i)
        return value
        
    # Integers
    def integer(self, tree: TypeTree):
        return ir.Constant(tree.ret, tree.children[0].value)

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
        return self.builder.extract_value(self.builder.smul_with_overflow(values[0], values[1]), 0)

    def negate_int(self, tree: TypeTree):
        value = self.visit_children(tree)[0]
        return self.builder.neg(value)

    def convert_int(self, tree: TypeTree):
        value = self.visit_children(tree)[0]
        return self.builder.sitofp(value, flt32)

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
        return ir.Constant(tree.ret, float(tree.children[0].value))

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
        return self.builder.fsub(make_flt(0), value)

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
