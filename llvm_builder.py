from lark.visitors import Interpreter
from llvmlite import ir

from util import TypeTree, h_bool, Lambda


class LLVMScope(Interpreter):
    def __init__(self, builder: ir.IRBuilder, arg_names: tuple, tree: TypeTree):
        self.builder = builder
        self.scope = {n: v for n, v in zip(arg_names, builder.function.args)}
        self.extra_args = self.builder.function.args[len(self.scope):]

        if tree.data == "code":
            self.visit_children(tree)
            if not builder.block.is_terminated:
                builder.ret(ir.Constant(tree.ret, 1))
        else:
            builder.ret(self.visit(tree))

    def assign(self, tree):
        name = tree.children[0].children[0].value
        self.scope[name] = self.visit(tree.children[1])
        return ir.Constant(h_bool, True)

    def symbol(self, tree: TypeTree):
        return self.scope[tree.children[0].value]

    def func(self, tree):
        if tree.children[0].data == "symbol":
            arg_names = (tree.children[0].children[0].value,)
        else:
            arg_names = (a.children[0].value for a in tree.children[0].children)

        module = self.builder.module
        f: ir.FunctionType = tree.ret
        ret = f.return_type
        args = f.args
        while isinstance(ret, ir.FunctionType):
            args = [*args, *ret.args]
            ret = ret.return_type
        f = ir.FunctionType(ret, args)
        func = ir.Function(module, f, str(next(module.func_count)))
        block = func.append_basic_block("entry")
        builder = ir.IRBuilder(block)

        LLVMScope(builder, arg_names, tree.children[1])
        return Lambda(func, ())

    def func_call(self, tree):
        args = self.visit(tree.children[1])
        if not isinstance(args, tuple):
            args = (args,)
        l: Lambda = self.scope[tree.children[0].value]
        args = (*l.args, *args)
        if len(args) < len(l.func.args):
            return Lambda(l.func, args)

        return self.builder.call(l.func, args)

    def returnn(self, tree):
        value = self.visit(tree.children[0])
        if isinstance(value, Lambda):
            args = (*value.args, *self.extra_args)
            if len(args) < len(value.func.args):
                value = Lambda(value.func, args)
            value = self.builder.call(value.func, args)

        self.builder.ret(value)
        return ir.Constant(h_bool, True)

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
