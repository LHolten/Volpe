from typing import Callable

from lark.visitors import Interpreter
from llvmlite import ir

from builder_utils import write_environment, read_environment, Closure, free_environment, environment_size, options
from util import TypeTree, int1, h_b


class LLVMScope(Interpreter):
    def __init__(self, builder: ir.IRBuilder, scope: dict, tree: TypeTree, ret: Callable):
        self.builder = builder
        self.scope = scope
        self.ret = ret

        if tree.data == "code":
            self.visit_children(tree)
            if not builder.block.is_terminated:
                assert False, "you forgot a return statement somewhere"
        else:
            ret(self.visit(tree))

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
            block = func.append_basic_block("entry")
            builder = ir.IRBuilder(block)

            env = func.args[0]
            env_values = read_environment(builder, env, env_types)
            args = dict(zip(env_names, env_values))
            args.update(dict(zip(arg_names, func.args[1:])))

            LLVMScope(builder, args, tree.children[1], builder.ret)
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

    def returnn(self, tree):
        value = self.visit(tree.children[0])

        environment = list(self.scope.values())
        print(self.scope.keys())
        if value in environment:
            print(list(self.scope.keys())[environment.index(value)])
            environment.remove(value)
        free_environment(self.builder, environment)

        self.ret(value)
        return ir.Constant(int1, True)

    def code(self, tree):
        phi = []

        with options(self.builder, tree.ret, phi) as ret:
            LLVMScope(self.builder, self.scope.copy(), tree, ret)

        return phi[0]

    def implication(self, tree):
        phi = []

        with options(self.builder, tree.ret, phi) as ret:
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(self.visit(tree.children[1]))
            ret(h_b(1))

        return phi[0]

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
