from lark.visitors import Interpreter
from llvmlite import ir

from builder_utils import ClosurePointer, write_environment, read_environment, Closure
from util import TypeTree, int1, int32, h, int8, pint8, h_b


class LLVMScope(Interpreter):
    def __init__(self, builder: ir.IRBuilder, args: dict, spots, tree: TypeTree):
        self.builder = builder
        self.scope = args
        self.spots = spots

        if tree.data == "code":
            self.visit_children(tree)
            if not builder.block.is_terminated:
                builder.ret(ir.Constant(tree.ret, 1))
        else:
            builder.ret(self.visit(tree))

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
        assert isinstance(f, ClosurePointer)

        values = list(self.scope.values())
        env_types = list(v.type for v in values)
        env_names = list(self.scope.keys())
        environment_ptr = self.builder.alloca(int8, h(f.size))
        write_environment(self.builder, environment_ptr, values)

        module = self.builder.module
        func = ir.Function(module, f.func, str(next(module.func_count)))
        block = func.append_basic_block("entry")
        builder = ir.IRBuilder(block)

        env = func.args[0]
        env_values = read_environment(builder, env, env_types)
        spots = func.args[1]
        args = dict(zip(env_names, env_values))
        args.update(dict(zip(arg_names, func.args[2:])))

        LLVMScope(builder, args, spots, tree.children[1])

        closure = ir.Constant(f, [func, f.reservation, f.size, ir.Undefined])
        closure = self.builder.insert_value(closure, environment_ptr, 3)
        return closure

    def func_call(self, tree):
        args = self.visit(tree.children[1])
        if not isinstance(args, tuple):
            args = (args,)

        l = self.scope[tree.children[0].value]
        assert isinstance(l.type, ClosurePointer)
        b = self.builder

        func_ptr = b.extract_value(l, 0)
        res_size = b.extract_value(l, 1)
        env_size = b.extract_value(l, 2)
        env_ptr = b.extract_value(l, 3)

        res_ptr = self.builder.alloca(int8, res_size)

        value = self.builder.call(func_ptr, [env_ptr, res_ptr, *args])

        if isinstance(value.type, Closure):
            new_func_ptr = self.builder.extract_value(value, 0)
            new_res_size = self.builder.extract_value(value, 1)

            closure = ir.Constant(ClosurePointer.from_closure(value.type, 0), ir.Undefined)
            closure = self.builder.insert_value(closure, new_func_ptr, 0)
            closure = self.builder.insert_value(closure, new_res_size, 1)
            closure = self.builder.insert_value(closure, res_size, 2)
            closure = self.builder.insert_value(closure, res_ptr, 3)

            return closure
        return value

    def returnn(self, tree):
        value = self.visit(tree.children[0])

        if isinstance(value.type, ClosurePointer):
            memcpy = self.builder.module.declare_intrinsic('llvm.memcpy', [pint8, pint8, int32])

            func_ptr = self.builder.extract_value(value, 0)
            res_size = self.builder.extract_value(value, 1)
            env_size = self.builder.extract_value(value, 2)
            env_ptr = self.builder.extract_value(value, 3)

            self.builder.call(memcpy, [self.spots, env_ptr, env_size, h_b(0)])

            closure = ir.Constant(Closure.from_closure_pointer(value.type), ir.Undefined)
            closure = self.builder.insert_value(closure, func_ptr, 0)
            closure = self.builder.insert_value(closure, res_size, 1)

            value = closure

        self.builder.ret(value)
        return ir.Constant(int1, True)

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
