from typing import Dict

from lark.visitors import Interpreter
from llvmlite import ir

from util import TypeTree, h_bool, Lambda


class LLVMScope(Interpreter):
    def __init__(self, builder: ir.IRBuilder, scope: Dict, tree: TypeTree):
        self.scope = scope
        self.builder = builder

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
        module = self.builder.module
        l: Lambda = tree.ret
        l.func = ir.Function(module, l.fntp, str(module.func_count))
        module.func_count += 1
        block = l.func.append_basic_block("entry")
        builder = ir.IRBuilder(block)

        l.scope = self.scope.copy()
        LLVMScope(builder, dict(zip(l.keys, l.func.args)), tree.children[1])
        return l

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

    def func_call(self, tree):
        args = self.visit(tree.children[1])
        if not isinstance(args, tuple):
            args = (args,)
        l: Lambda = self.scope[tree.children[0].value]
        l.scope.update(dict(zip(l.arg_keys, args)))

        return self.builder.call(l.func, [l.scope[k] for k in l.keys])

    def returnn(self, tree):
        self.builder.ret(self.visit(tree.children[0]))
        return ir.Constant(h_bool, True)

    def __default__(self, tree):
        raise NotImplementedError("llvm", tree.data)
