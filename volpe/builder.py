from typing import Callable

from lark.visitors import Interpreter
from llvmlite import ir

from builder_utils import write_environment, free_environment, options, \
    read_environment, tuple_assign, copy, copy_environment, build_closure, closure_call, free
from tree import TypeTree
from volpe_types import int1, flt32, flt64, VolpeClosure, int32, target_data


class LLVMScope(Interpreter):
    flt = flt64

    def __init__(self, builder: ir.IRBuilder, tree: TypeTree, scope: dict, ret: Callable):
        self.builder = builder
        self.scope = scope
        self.ret = ret

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
        closure_type = tree.return_type
        assert isinstance(closure_type, VolpeClosure)

        module = self.builder.module
        env_names = list(closure_type.outside_used)
        env_values = [self.scope[name] for name in env_names]
        env_types = [value.type for value in env_values]

        with build_closure(module, closure_type, env_types) as (b, args, closure, c_func):
            if closure_type.checked:
                new_values = copy_environment(b, read_environment(b, args[0], env_types))

                new_scope = dict(zip(env_names, new_values))
                for shape, value in zip(tree.children[:-1], args[1:]):
                    tuple_assign(b, new_scope, shape, value)
                new_scope["@"] = b.insert_value(closure, b.call(c_func, [args[0]]), 3)

                # Creates new LLVMScope or FastLLVMScope.
                self.__class__(b, tree.children[-1], new_scope, b.ret)

            else:
                b.ret_void()
                print("ignoring function without usage")

        env_ptr = write_environment(self.builder, copy_environment(self.builder, env_values))
        return self.builder.insert_value(closure, env_ptr, 3)

    def func_call(self, tree: TypeTree):
        values = self.visit_children(tree)
        closure = values[0]
        args = values[1:]

        res = closure_call(self.builder, closure, args)
        free(self.builder, closure)
        return res

    def this_func(self, tree: TypeTree):
        return copy(self.builder, self.scope["@"])

    def return_n(self, tree: TypeTree):
        value = self.visit(tree.children[0])
        free_environment(self.builder, self.scope.values())
        self.ret(value)

    def block(self, tree: TypeTree):
        new_scope = dict(zip(self.scope.keys(), copy_environment(self.builder, self.scope.values())))
        with options(self.builder, tree.return_type) as (ret, phi):
            # Creates new LLVMScope or FastLLVMScope.
            self.__class__(self.builder, tree, new_scope, ret)
        return phi

    def number_iter(self, tree: TypeTree):
        values = self.visit_children(tree)
        closure_type = tree.return_type.closure
        module = self.builder.module
        env_types = [int32]

        # TODO compact or simplify the following code:

        with options(self.builder, tree.return_type) as (ret, phi):
            # reverse = a > b in (a..b).
            reverse = self.builder.icmp_signed(">", values[0], values[1])
            with self.builder.if_then(reverse):
                # Going in reverse (10..0).
                with build_closure(module, closure_type, env_types) as (b, args, closure, c_func):
                    start = read_environment(b, args[0], env_types)[0]
                    b.ret(b.sub(start, b.add(args[1], int32(1))))

                env_ptr = write_environment(self.builder, [values[0]])
                list_value = tree.return_type(ir.Undefined)
                list_value = self.builder.insert_value(list_value, self.builder.insert_value(closure, env_ptr, 3), 0)
                ret(self.builder.insert_value(list_value, self.builder.sub(values[0], values[1]), 1))

            # Going forwards (0..10).
            with build_closure(module, closure_type, env_types) as (b, args, closure, c_func):
                start = read_environment(b, args[0], env_types)[0]
                b.ret(b.add(start, args[1]))

            env_ptr = write_environment(self.builder, [values[0]])
            list_value = tree.return_type(ir.Undefined)
            list_value = self.builder.insert_value(list_value, self.builder.insert_value(closure, env_ptr, 3), 0)
            ret(self.builder.insert_value(list_value, self.builder.sub(values[1], values[0]), 1))

        return phi

    def list_index(self, tree: TypeTree):
        list_value, i = self.visit_children(tree)
        pointer = self.builder.extract_value(list_value, 0)
        length = self.builder.extract_value(list_value, 1)

        before_end = self.builder.icmp_signed("<", i, length)
        more_than_0 = self.builder.icmp_signed(">=", i, int32(0))
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

    def map(self, tree: TypeTree):
        values = self.visit_children(tree)
        closure_type = tree.return_type.closure
        module = self.builder.module
        env_types = [values[0].type.closure, values[1].type]

        with build_closure(module, closure_type, env_types) as (b, args, closure, c_func):
            closure1, closure2 = read_environment(b, args[0], env_types)
            temp = closure_call(b, closure1, [args[1]])
            b.ret(closure_call(b, closure2, [temp]))

        closure1 = self.builder.extract_value(values[0], 0)
        env_ptr = write_environment(self.builder, [closure1, values[1]])
        list_value = tree.return_type(ir.Undefined)
        list_value = self.builder.insert_value(list_value, self.builder.insert_value(closure, env_ptr, 3), 0)
        return self.builder.insert_value(list_value, self.builder.extract_value(values[0], 1), 1)

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
            self.builder.store(closure_call(self.builder, closure, [phi]), self.builder.gep(pointer, [phi]))
            ret(self.builder.add(phi, int32(1)))

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

    def object(self, tree: TypeTree):
        value = tree.return_type(ir.Undefined)
        for i, v in enumerate(self.visit_children(tree)):
            value = self.builder.insert_value(value, v, i)
        return value
        
    # Integers
    def integer(self, tree: TypeTree):
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
        float_value = self.builder.sitofp(value, self.flt)
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
        return self.builder.fsub(self.flt(0), value)

    def convert_flt(self, tree: TypeTree):
        value = self.visit_children(tree)[0]
        return self.builder.fptosi(value, int32)

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


class FastLLVMScope(LLVMScope):
    flt = flt32