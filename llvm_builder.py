import itertools

from lark.visitors import Interpreter
from llvmlite import ir

from util import TypeTree, h_bool, Lambda, LambdaAnnotation, h_int, h, target_data, h_byte


class LLVMScope(Interpreter):
    def __init__(self, builder: ir.IRBuilder, args: tuple, arg_names: tuple, env_names: tuple, tree: TypeTree):
        self.builder = builder
        if len(args) > 0:
            self.spots = args[1]
            self.scope = {k: builder.load(builder.gep(args[0], (h(0), h(i)))) for i, k in enumerate(env_names)}
            self.scope.update(dict(zip(arg_names, args[2])))
        else:
            self.scope = {}

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

        f = tree.ret  # function type
        assert isinstance(f, LambdaAnnotation)

        env_struct = ir.Constant(f.env, ir.Undefined)
        for i, k in enumerate(f.env_names):
            env_struct = self.builder.insert_value(env_struct, self.scope[k], i)
        env_struct = self.builder.bitcast(env_struct, ir.ArrayType(h_byte, env_struct.type.get_abi_size(target_data)))

        fnt = f.elements[0]
        module = self.builder.module
        func = ir.Function(module, fnt.pointee, str(next(module.func_count)))
        block = func.append_basic_block("entry")
        builder = ir.IRBuilder(block)

        env = builder.bitcast(func.args[0], f.env.as_pointer())
        spots = func.args[1]
        args = func.args[2:]

        LLVMScope(builder, (env, spots, args), arg_names, f.env_names, tree.children[1])

        lamb = ir.Constant(ir.LiteralStructType((fnt, env_struct.type)), ir.Undefined)
        lamb = self.builder.insert_value(lamb, func, 0)
        lamb = self.builder.insert_value(lamb, env_struct, 1)

        return lamb

    def func_call(self, tree):
        args = self.visit(tree.children[1])
        if not isinstance(args, tuple):
            args = (args,)

        l = self.scope[tree.children[0].value]
        f: LambdaAnnotation = tree.func

        func = self.builder.extract_value(l, 0)
        env = self.builder.extract_value(l, 1)

        env_loc = self.builder.alloca(env.type)
        self.builder.store(env, env_loc)
        env_loc_not_type = self.builder.bitcast(env_loc, h_byte.as_pointer())

        spot_type = ir.VectorType(h_byte, f.spot_size)
        spot_loc = self.builder.alloca(spot_type)  # reserve new env_loc
        spot_loc_not_type = self.builder.bitcast(spot_loc, h_byte.as_pointer())

        args = (env_loc_not_type, spot_loc_not_type, *args)
        value = self.builder.call(func, args)

        if isinstance(f.return_type, ir.PointerType):  # needs to be better
            lamb = ir.Constant(ir.LiteralStructType((value.type, spot_type)), ir.Undefined)
            lamb = self.builder.insert_value(lamb, value, 0)
            lamb = self.builder.insert_value(lamb, self.builder.load(spot_loc), 1)

            return lamb
        return value

    def returnn(self, tree):
        value = self.visit(tree.children[0])

        if isinstance(value.type, ir.LiteralStructType):  # needs to be better
            env = self.builder.extract_value(value, 1)
            spots = self.builder.bitcast(self.spots, env.type.as_pointer())

            self.builder.store(env, spots)

            value = self.builder.extract_value(value, 0)

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
