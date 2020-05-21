from typing import Callable, Optional, Tuple

from lark.visitors import Interpreter
from llvmlite import ir

from builder_utils import write_environment, free_environment, options, \
    read_environment, tuple_assign, copy, copy_environment, build_closure, free, math, comp, unary_math, \
    check_list_index
from tree import TypeTree, volpe_assert
from volpe_types import int1, int64, flt64, target_data, pint8, unwrap, VolpeObject


class LLVMScope(Interpreter):
    def __init__(self, builder: ir.IRBuilder, tree: TypeTree, scope: callable, ret: Callable, rec: Optional[Callable],
                 args: Optional[Tuple[TypeTree, VolpeObject]] = None):
        self.builder = builder
        self.scope = scope
        self.local_scope = dict()
        if args is not None:
            tuple_assign(self, *args)
        self.ret = ret
        self.rec = rec

        def evaluate(children):
            if len(children) == 1:
                self.visit_unsafe(children[0])
            else:
                success = self.visit(children[0])
                with self.builder.if_then(success):
                    evaluate(children[1:])
                builder.unreachable()

        evaluate(tree.children)

    def get_scope(self, name, mut):
        if name not in self.local_scope:
            if not mut:
                return self.scope(name, False)
            self.local_scope[name] = copy(self.builder, self.scope(name, False))
        return self.local_scope[name]

    def free(self, value):
        if not value.tracked:
            free(self.builder, value)

    def copy(self, value):
        if value.tracked:
            value = copy(self.builder, value)
            value.tracked = False
        return value

    def visit(self, tree: TypeTree):
        value = getattr(self, tree.data)(tree)
        volpe_assert(not self.builder.block.is_terminated, "dead code is not allowed", tree)
        value.tracked = getattr(value, "tracked", False)
        return value

    def visit_unsafe(self, tree: TypeTree):
        return getattr(self, tree.data)(tree)

    def assign(self, tree: TypeTree):
        tuple_assign(self, tree.children[0], self.copy(self.visit(tree.children[1])))
        return int1(True)

    def symbol(self, tree: TypeTree):
        value = self.get_scope(tree.children[0].value, False)
        value.tracked = True
        return value

    def func(self, tree: TypeTree):
        closure_type = tree.return_type

        module = self.builder.module
        env_names = list(tree.outside_used)
        env_values = [self.get_scope(name, False) for name in env_names]
        env_types = [value.type for value in env_values]

        with build_closure(module, closure_type, env_types) as (b, rec, args, closure):
            new_values = read_environment(b, args[0], env_types)

            env_scope = dict(zip(env_names, new_values))
            env_scope["@"] = b.insert_value(closure, args[0], 3)

            def scope(name, mut):
                assert not mut, "this scope can't be mutated"
                return env_scope[name]

            LLVMScope(b, tree.children[1], scope, b.ret, rec, (tree.children[0], args[1]))

        env_ptr = write_environment(self.builder, copy_environment(self.builder, env_values))
        return self.builder.insert_value(closure, env_ptr, 3)

    def func_call(self, tree: TypeTree):
        closure, args = self.visit_children(tree)

        func = self.builder.extract_value(closure, 0)
        env_ptr = self.builder.extract_value(closure, 3)

        res = self.builder.call(func, [env_ptr, args])

        self.free(closure)
        return res

    def return_n(self, tree: TypeTree):
        # recursive tail call
        if tree.children[0].data == "func_call" \
                and tree.children[0].children[0].data == "symbol" \
                and tree.children[0].children[0].children[0].value == "@" \
                and self.rec is not None:  # prevent tail call optimization in blocks
            value = self.copy(self.visit(tree.children[0].children[1]))
            free_environment(self.builder, self.local_scope.values())
            self.rec(value)
            return int1

        value = self.copy(self.visit(tree.children[0]))
        free_environment(self.builder, self.local_scope.values())
        self.ret(value)
        return int1

    def block(self, tree: TypeTree):
        with options(self.builder, unwrap(tree.return_type)) as (ret, phi):
            self.__class__(self.builder, tree, self.get_scope, ret, None)
        return phi

    def object(self, tree: TypeTree):
        value = unwrap(tree.return_type)(ir.Undefined)
        for i, child in enumerate(tree.children):
            value = self.builder.insert_value(value, self.copy(self.visit(child)), i)
        return value

    def list_index(self, tree: TypeTree):
        list_value, i = self.visit_children(tree)
        check_list_index(self.builder, list_value, i)
        pointer = self.builder.extract_value(list_value, 0)

        res = self.builder.load(self.builder.gep(pointer, [i]))
        self.free(list_value)
        res.tracked = list_value.tracked
        return res

    def list_size(self, tree: TypeTree):
        list_value = self.visit_children(tree)[0]
        length = self.builder.extract_value(list_value, 1)
        self.free(list_value)
        return length

    def list(self, tree: TypeTree):
        element_type = unwrap(tree.return_type.element_type)
        data_size = int64(element_type.get_abi_size(target_data) * len(tree.children))
        pointer = self.builder.call(self.builder.module.malloc, [data_size])
        pointer = self.builder.bitcast(pointer, element_type.as_pointer())

        for i, ret in enumerate(self.visit_children(tree)):
            self.builder.store(self.copy(ret), self.builder.gep(pointer, [int64(i)]))

        list_value = unwrap(tree.return_type)(ir.Undefined)
        list_value = self.builder.insert_value(list_value, pointer, 0)
        return self.builder.insert_value(list_value, int64(len(tree.children)), 1)

    def concatenate(self, tree: TypeTree):
        list_value, other_list = self.visit_children(tree)
        list_value = self.copy(list_value)
        other_list = self.copy(other_list)

        element_type = unwrap(tree.return_type.element_type)
        data_size = int64(element_type.get_abi_size(target_data))

        b = self.builder
        pointer = b.bitcast(b.extract_value(list_value, 0), pint8)
        length = b.mul(b.extract_value(list_value, 1), data_size)
        pointer2 = b.bitcast(b.extract_value(other_list, 0), pint8)
        length2 = b.mul(b.extract_value(other_list, 1), data_size)

        new_length = b.add(length, length2)
        new_pointer = b.call(self.builder.module.malloc, [new_length])
        b.call(b.module.memcpy, [new_pointer, pointer, length, int1(False)])
        b.call(b.module.memcpy, [b.gep(new_pointer, [length]), pointer2, length2, int1(False)])
        b.call(b.module.free, [pointer])
        b.call(b.module.free, [pointer2])

        new_list = unwrap(tree.return_type)(ir.Undefined)
        new_list = self.builder.insert_value(new_list, b.bitcast(new_pointer, element_type.as_pointer()), 0)
        new_size = b.sdiv(new_length, data_size)
        return self.builder.insert_value(new_list, new_size, 1)

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
        # let Python parse the escaped character
        evaluated = eval(f"{tree.children[0]}")
        return tree.return_type(ord(evaluated))

    # Mathematics
    add = math
    mod = math
    mul = math
    sub = math
    div = math
    # power = math
    negate = unary_math

    # Comparison
    equals = comp
    not_equals = comp
    greater = comp
    less = comp
    greater_equals = comp
    less_equals = comp

    def __default__(self, tree: TypeTree):
        raise NotImplementedError("llvm", tree.data)
