from typing import Callable

from lark.visitors import Interpreter
from llvmlite import ir

from builder_utils import write_environment, free_environment, options, \
    read_environment, tuple_assign, copy, copy_environment, build_closure, free
from tree import TypeTree
from volpe_types import int1, int32, int64, flt64, VolpeClosure, target_data


class LLVMScope(Interpreter):
    def __init__(self, builder: ir.IRBuilder, tree: TypeTree, scope: callable, ret: Callable, rec: Callable):
        self.builder = builder
        self.scope = scope
        self.local_scope = dict()
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

    def get_scope(self, name):
        if name in self.local_scope:
            return copy(self.builder, self.local_scope[name])
        return self.scope(name)

    def visit(self, tree: TypeTree):
        value = getattr(self, tree.data)(tree)
        assert not self.builder.block.is_terminated, "dead code is not allowed"
        return value

    def visit_unsafe(self, tree: TypeTree):
        return getattr(self, tree.data)(tree)

    def assign(self, tree: TypeTree):
        tuple_assign(self.builder, self.local_scope, tree.children[0], self.visit(tree.children[1]))
        return int1(True)

    def symbol(self, tree: TypeTree):
        return self.get_scope(tree.children[0].value)

    def func(self, tree: TypeTree):
        closure_type = tree.return_type
        assert isinstance(closure_type, VolpeClosure)

        module = self.builder.module
        env_names = list(closure_type.outside_used)
        env_values = [self.get_scope(name) for name in env_names]
        env_types = [value.type for value in env_values]

        with build_closure(module, closure_type, env_types) as (b, rec_ret, args, closure, c_func):
            if closure_type.checked:
                new_values = read_environment(b, args[0], env_types)

                env_scope = dict(zip(env_names, new_values))
                env_scope["@"] = b.insert_value(closure, args[0], 3)

                arg_scope = tuple_assign(b, dict(), tree.children[0], args[1])

                def scope(name):
                    if name in arg_scope:
                        return copy(b, arg_scope[name])
                    return copy(b, env_scope[name])

                def ret(value):
                    free_environment(b, arg_scope.values())
                    b.ret(value)

                def rec(value):
                    free_environment(b, arg_scope.values())
                    rec_ret(value)

                self.__class__(b, tree.children[-1], scope, ret, rec)
            else:
                b.ret_void()
                print("ignoring function without usage")

        env_ptr = write_environment(self.builder, copy_environment(self.builder, env_values))
        return self.builder.insert_value(closure, env_ptr, 3)

    def func_call(self, tree: TypeTree):
        values = self.visit_children(tree)
        closure = values[0]
        args = values[1]

        func = self.builder.extract_value(closure, 0)
        env_ptr = self.builder.extract_value(closure, 3)

        res = self.builder.call(func, [env_ptr, args])

        free(self.builder, closure)
        return res

    def return_n(self, tree: TypeTree):
        # recursive tail call
        if tree.children[0].data == "func_call" \
                and tree.children[0].children[0].data == "symbol" \
                and tree.children[0].children[0].children[0].value == "@" \
                and self.rec is not None:  # prevent tail call optimization in blocks
            value = self.visit(tree.children[0].children[1])
            free_environment(self.builder, self.local_scope.values())
            self.rec(value)
            return int1

        value = self.visit(tree.children[0])
        free_environment(self.builder, self.local_scope.values())
        self.ret(value)
        return int1

    def block(self, tree: TypeTree):
        with options(self.builder, tree.return_type) as (ret, phi):
            self.__class__(self.builder, tree, self.get_scope, ret, None)
        return phi

    def object(self, tree: TypeTree):
        value = tree.return_type(ir.Undefined)
        for i, child in enumerate(tree.children):
            value = self.builder.insert_value(value, self.visit(child), i)
        return value

    def list_index(self, tree: TypeTree):
        list_value, i = self.visit_children(tree)
        pointer = self.builder.extract_value(list_value, 0)
        length = self.builder.extract_value(list_value, 1)

        before_end = self.builder.icmp_signed("<", i, length)
        more_than_0 = self.builder.icmp_signed(">=", i, int64(0))
        in_range = self.builder.and_(before_end, more_than_0)
        with self.builder.if_then(self.builder.not_(in_range)):
            self.builder.unreachable()

        res = self.builder.load(self.builder.gep(pointer, [i]))
        free(self.builder, list_value)
        return res

    def list_size(self, tree: TypeTree):
        list_value = self.visit_children(tree)[0]
        length = self.builder.extract_value(list_value, 1)
        free(self.builder, list_value)
        return length

    def make_list(self, tree: TypeTree):
        iter_value = self.visit_children(tree)[0]
        closure = self.builder.extract_value(iter_value, 0)
        length = self.builder.extract_value(iter_value, 1)
        data_size = int32(tree.return_type.element_type.get_abi_size(target_data))
        pointer = self.builder.call(self.builder.module.malloc, [self.builder.mul(data_size, length)])
        pointer = self.builder.bitcast(pointer, tree.return_type.element_type.as_pointer())

        with options(self.builder, int32) as (ret, phi):
            ret(int32(0))

        with self.builder.if_then(self.builder.icmp_signed("<", phi, length)):
            # TODO fix lists
            # self.builder.store(closure_call(self.builder, closure, [phi]), self.builder.gep(pointer, [phi]))
            ret(self.builder.add(phi, int64(1)))

        free(self.builder, closure)

        list_value = tree.return_type(ir.Undefined)
        list_value = self.builder.insert_value(list_value, pointer, 0)
        return self.builder.insert_value(list_value, length, 1)

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
        value = self.visit(tree.children[0])
        float_value = self.builder.sitofp(value, flt64)
        decimals = tree.return_type(float("0." + tree.children[1].value))
        return self.builder.fadd(float_value, decimals)

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
        return self.builder.fsub(flt64(0), value)

    def convert_flt(self, tree: TypeTree):
        value = self.visit_children(tree)[0]
        return self.builder.fptosi(value, int64)

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

    @staticmethod
    def character(tree: TypeTree):
        return tree.return_type(ord(tree.children[0].value[1]))

    @staticmethod
    def escaped_character(tree: TypeTree):
        # let Python parse the escaped character
        evaluated = eval(f"{tree.children[0]}")
        return tree.return_type(ord(evaluated))

    def __default__(self, tree: TypeTree):
        raise NotImplementedError("llvm", tree.data)
