from typing import Callable, Optional, Tuple

from lark.visitors import Interpreter
from llvmlite import ir

from builder_utils import options, assign, math, comp, build_or_get_function
from tree import TypeTree, volpe_assert, get_obj_key_value
from volpe_types import int1, int64, flt64, unwrap, VolpeObject


class LLVMScope(Interpreter):
    def __init__(
        self,
        builder: ir.IRBuilder,
        tree: TypeTree,
        scope: callable,
        ret: Callable,
        rec: Optional[Callable],
        args: Optional[Tuple[TypeTree, VolpeObject]] = None,
    ):
        self.builder = builder
        self.scope = scope
        self.local_scope = dict()
        if args is not None:
            assign(self, *args)
        self.ret = ret
        self.rec = rec

        def evaluate(children):
            if len(children) == 1:
                self.visit_unsafe(children[0])
            else:
                self.visit(children[0])
                evaluate(children[1:])

        evaluate(tree.children)

    def get_scope(self, name):
        if name not in self.local_scope:
            return self.scope(name)
        return self.local_scope[name]

    def visit(self, tree: TypeTree, safe=True):
        value = getattr(self, tree.data)(tree)
        if safe:
            volpe_assert(not self.builder.block.is_terminated, "dead code is not allowed", tree)
        return value

    def visit_unsafe(self, tree: TypeTree):
        return getattr(self, tree.data)(tree)

    def assign(self, tree: TypeTree):
        value = self.visit(tree.children[1])
        assign(self, tree.children[0], value)
        return int1(True)

    def symbol(self, tree: TypeTree):
        value = self.get_scope(tree.children[0].value)
        return value

    def func(self, tree: TypeTree):
        closure = unwrap(tree.return_type)(ir.Undefined)
        for i, name in enumerate(tree.return_type.env.keys()):
            closure = self.builder.insert_value(closure, self.get_scope(name), i)
        return closure

    def func_call(self, tree: TypeTree):
        closure, args = self.visit_children(tree)
        func = build_or_get_function(self, tree)
        return self.builder.call(func, [closure, args])

    def return_n(self, tree: TypeTree):
        if (
            tree.children[0].data == "func_call"
            and tree.children[0].children[0].data == "symbol"
            and tree.children[0].children[0].children[0].value == "@"
            and self.rec is not None
        ):  # prevent tail call optimization in blocks
            self.rec(self.visit(tree.children[0].children[1]))
        else:
            self.ret(self.visit(tree.children[0]))

    def block(self, tree: TypeTree):
        with options(self.builder, unwrap(tree.return_type)) as (ret, phi):
            self.__class__(self.builder, tree, self.get_scope, ret, None)
        return phi

    def object(self, tree: TypeTree):
        value = unwrap(tree.return_type)(ir.Undefined)
        for i, child in enumerate(tree.children):
            key, attribute = get_obj_key_value(child, i)
            value = self.builder.insert_value(value, self.visit(attribute), i)
        return value

    def attribute(self, tree: TypeTree):
        value = self.visit(tree.children[0])
        index = list(value.type.type_dict.keys()).index(tree.children[1])
        return self.builder.extract_value(value, index)

    def list_index(self, tree: TypeTree):
        array_value, i = self.visit_children(tree)
        return self.builder.extract_element(array_value, i)

    @staticmethod
    def list_size(tree: TypeTree):
        return int64(tree.children[0].return_type.count)

    def list(self, tree: TypeTree):
        array_value = unwrap(tree.return_type)(ir.Undefined)
        for i, ret in enumerate(self.visit_children(tree)):
            array_value = self.builder.insert_element(array_value, ret, int64(i))
        return array_value

    def implication(self, tree: TypeTree):
        with options(self.builder, tree.return_type) as (ret, phi):
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(self.visit_unsafe(tree.children[1]))
            ret(int1(True))

        return phi

    def logic_and(self, tree: TypeTree):
        with options(self.builder, tree.return_type) as (ret, phi):
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(self.visit_unsafe(tree.children[1]))
            ret(int1(False))

        return phi

    def logic_or(self, tree: TypeTree):
        with options(self.builder, tree.return_type) as (ret, phi):
            value = self.visit(tree.children[0])
            with self.builder.if_then(value):
                ret(int1(True))
            ret(self.visit_unsafe(tree.children[1]))

        return phi

    def logic_not(self, tree: TypeTree):
        value = self.visit_children(tree)[0]
        return self.builder.not_(value)

    # Integers
    @staticmethod
    def integer(tree: TypeTree):
        return tree.return_type(int(tree.children[0].value))

    def add_int(self, values):
        # TODO Use overflow bit to raise runtime error
        # self.builder.extract_value(self.builder.sadd_with_overflow(values[0], values[1]), 0)
        return self.builder.add(values[0], values[1])

    def sub_int(self, values):
        # TODO Use overflow bit to raise runtime error
        # self.builder.extract_value(self.builder.ssub_with_overflow(values[0], values[1]), 0)
        return self.builder.sub(values[0], values[1])

    def mod_int(self, values):
        return self.builder.srem(values[0], values[1])

    def div_int(self, values):
        return self.builder.sdiv(values[0], values[1])

    def mul_int(self, values):
        # TODO Use overflow bit to raise runtime error
        # self.builder.extract_value(self.builder.smul_with_overflow(values[0], values[1]), 0)
        return self.builder.mul(values[0], values[1])

    def negate_int(self, values):
        return self.builder.neg(values[0])

    def convert_int(self, tree: TypeTree):
        value = self.visit(tree.children[0])
        float_value = self.builder.sitofp(value, flt64)
        decimals = tree.return_type(float("0." + tree.children[1].value))
        return self.builder.fadd(float_value, decimals)

    def equals_int(self, values):
        return self.builder.icmp_signed("==", values[0], values[1])

    def not_equals_int(self, values):
        return self.builder.icmp_signed("!=", values[0], values[1])

    def greater_int(self, values):
        return self.builder.icmp_signed(">", values[0], values[1])

    def less_int(self, values):
        return self.builder.icmp_signed("<", values[0], values[1])

    def greater_equals_int(self, values):
        return self.builder.icmp_signed(">=", values[0], values[1])

    def less_equals_int(self, values):
        return self.builder.icmp_signed("<=", values[0], values[1])

    def add_flt(self, values):
        return self.builder.fadd(values[0], values[1])

    def sub_flt(self, values):
        return self.builder.fsub(values[0], values[1])

    def mod_flt(self, values):
        return self.builder.frem(values[0], values[1])

    def div_flt(self, values):
        return self.builder.fdiv(values[0], values[1])

    def mul_flt(self, values):
        return self.builder.fmul(values[0], values[1])

    def negate_flt(self, values):
        return self.builder.fsub(flt64(0), values[0])

    def convert_flt(self, tree: TypeTree):
        value = self.visit_children(tree)[0]
        return self.builder.fptosi(value, int64)

    def equals_flt(self, values):
        return self.builder.fcmp_ordered("==", values[0], values[1])

    def not_equals_flt(self, values):
        return self.builder.fcmp_ordered("!=", values[0], values[1])

    def greater_flt(self, values):
        return self.builder.fcmp_ordered(">", values[0], values[1])

    def less_flt(self, values):
        return self.builder.fcmp_ordered("<", values[0], values[1])

    def greater_equals_flt(self, values):
        return self.builder.fcmp_ordered(">=", values[0], values[1])

    def less_equals_flt(self, values):
        return self.builder.fcmp_ordered("<=", values[0], values[1])

    @staticmethod
    def character(tree: TypeTree):
        return tree.return_type(ord(tree.children[0].value[1]))

    @staticmethod
    def escaped_character(tree: TypeTree):
        # let Python parse the escaped character (guaranteed ascii by lark)
        evaluated = eval(f"{tree.children[0]}")
        return tree.return_type(ord(evaluated))

    # Mathematics
    add = math
    mod = math
    mul = math
    sub = math
    div = math
    # power = math
    negate = math

    # Comparison
    equals = comp
    not_equals = comp
    greater = comp
    less = comp
    greater_equals = comp
    less_equals = comp

    def __default__(self, tree: TypeTree):
        raise NotImplementedError("llvm", tree.data)
