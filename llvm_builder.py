from typing import Callable

from lark.visitors import Interpreter
from llvmlite import ir

from builder_utils import write_environment, Closure, free_environment, environment_size, options, \
    read_environment
from util import TypeTree, int1, h_b, pint8, int32


class LLVMScope(Interpreter):
    def __init__(self, builder: ir.IRBuilder, scope: dict, tree: TypeTree, ret: Callable, old_scope: set, closure: Closure):
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
            ret(self.visit(tree))

    def visit(self, tree):
        value = getattr(self, tree.data)(tree)
        assert not self.builder.block.is_terminated, "dead code is not allowed"
        return value

    def visit_unsafe(self, tree):
        return getattr(self, tree.data)(tree)

    def assign(self, tree):
        name = tree.children[0].children[0].value
        self.scope[name] = self.visit(tree.children[1])
        return ir.Constant(int1, True)

    def symbol(self, tree: TypeTree):
        return self.scope[tree.children[0].value]

    def func(self, tree):
        if tree.children[0].data == "symbol":
            arg_names = (tree.children[0].children[0].value,)
        else:
            arg_names = (a.children[0].value for a in tree.children[0].children)

        f = tree.ret
        assert isinstance(f, Closure)

        values = list(self.scope.values())
        env_types = list(v.type for v in values)
        env_names = list(self.scope.keys())

        module = self.builder.module
        env_size = environment_size(self.builder, values)
        env_ptr = self.builder.call(module.malloc, [env_size])
        write_environment(self.builder, env_ptr, values)

        func = ir.Function(module, f.func, str(next(module.func_count)))

        if f.func.args:
            build_function(func, env_names, env_types, arg_names, tree.children[1])
        else:
            print("ignoring function without usage")

        closure = ir.Constant(f, [func, ir.Undefined, ir.Undefined])
        closure = self.builder.insert_value(closure, env_size, 1)
        closure = self.builder.insert_value(closure, env_ptr, 2)
        return closure

    def func_call(self, tree):
        args = self.visit(tree.children[1])
        if not isinstance(args, tuple):
            args = (args,)

        closure = self.visit(tree.children[0])
        assert isinstance(closure.type, Closure)

        func_ptr = self.builder.extract_value(closure, 0)
        env_size = self.builder.extract_value(closure, 1)
        env_ptr = self.builder.extract_value(closure, 2)

        return self.builder.call(func_ptr, [env_ptr, *args])

    def this_func(self, tree: TypeTree):
        return self.closure

    def returnn(self, tree):
        value = self.visit(tree.children[0])

        scope = set(self.scope.values()) - {value} - self.old_scope
        free_environment(self.builder, scope)

        self.ret(value)

    def code(self, tree):
        phi = []

        with options(self.builder, tree.ret, phi) as ret:
            LLVMScope(self.builder, self.scope.copy(), tree, ret, set(self.scope.values()), self.closure)

        return phi[0]

    def implication(self, tree):
        phi = []

        with options(self.builder, tree.ret, phi) as ret:
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(self.visit_unsafe(tree.children[1]))
            ret(h_b(1))

        return phi[0]

    def logic_and(self, tree):
        phi = []

        with options(self.builder, tree.ret, phi) as ret:
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(self.visit_unsafe(tree.children[1]))
            ret(h_b(0))

        return phi[0]

    def logic_or(self, tree):
        phi = []

        with options(self.builder, tree.ret, phi) as ret:
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(h_b(1))
            ret(self.visit_unsafe(tree.children[1]))

        return phi[0]

    def logic_not(self, tree):
        value = self.visit_children(tree)[0]
        return self.builder.not_(value)

    def tuple(self, tree):
        return tuple(self.visit_children(tree))

    def number(self, tree):
        return ir.Constant(tree.ret, tree.children[0].value)

    def add(self, tree):
        values = self.visit_children(tree)
        return self.builder.add(values[0], values[1])

    def sub(self, tree):
        values = self.visit_children(tree)
        return self.builder.sub(values[0], values[1])

    def mod(self, tree):
        values = self.visit_children(tree)
        return self.builder.srem(values[0], values[1])

    def div(self, tree):
        values = self.visit_children(tree)
        return self.builder.sdiv(values[0], values[1])

    def mul(self, tree):
        values = self.visit_children(tree)
        return self.builder.extract_value(self.builder.smul_with_overflow(values[0], values[1]), 0)

    def negate(self, tree):
        value = self.visit_children(tree)[0]
        return self.builder.neg(value)

    def equals(self, tree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed("==", values[0], values[1])

    def not_equals(self, tree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed("!=", values[0], values[1])

    def greater(self, tree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed(">", values[0], values[1])

    def less(self, tree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed("<", values[0], values[1])

    def greater_equals(self, tree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed(">=", values[0], values[1])

    def less_equals(self, tree):
        values = self.visit_children(tree)
        return self.builder.icmp_signed("<=", values[0], values[1])

    def __default__(self, tree):
        raise NotImplementedError("llvm", tree.data)


def build_function(func: ir.Function, env_names, env_types, arg_names, code):
    block = func.append_basic_block("entry")
    builder = ir.IRBuilder(block)

    env = func.args[0]
    env_values = read_environment(builder, env, env_types)
    args = dict(zip(env_names, env_values))
    args.update(dict(zip(arg_names, func.args[1:])))

    this_env_size = environment_size(builder, env_values)
    closure = ir.Constant(Closure(func.type), [func, ir.Undefined, ir.Undefined])
    closure = builder.insert_value(closure, this_env_size, 1)
    closure = builder.insert_value(closure, env, 2)

    LLVMScope(builder, args, code, builder.ret, set(), closure)
