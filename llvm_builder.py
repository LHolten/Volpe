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

        env_loc = self.builder.alloca(f.env)  # reserve new env_loc
        list(self.builder.store(self.scope[k], self.builder.gep(env_loc, (h(0), h(i)))) for i, k in enumerate(f.env_names))

        fnt, env_loc_t, spt = f.elements
        module = self.builder.module
        func = ir.Function(module, fnt.pointee, str(next(module.func_count)))
        block = func.append_basic_block("entry")
        builder = ir.IRBuilder(block)

        env = builder.bitcast(func.args[0], f.env.as_pointer())
        spots = builder.bitcast(func.args[1], f.spots.as_pointer())
        args = func.args[2:]

        LLVMScope(builder, (env, spots, args), arg_names, f.env_names, tree.children[1])

        l = ir.Constant(ir.LiteralStructType((func.type, h_byte.as_pointer(), h_int)), (func, ir.Undefined, h(f.spots.get_abi_size(target_data))))
        l = self.builder.insert_value(l, self.builder.bitcast(env_loc, h_byte.as_pointer()), 1)
        l.spot_id = f.spot_id

        return l

    def func_call(self, tree):
        args = self.visit(tree.children[1])
        if not isinstance(args, tuple):
            args = (args,)

        l = self.scope[tree.children[0].value]
        func = self.builder.extract_value(l, 0)
        env_loc = self.builder.extract_value(l, 1)
        spot_size = self.builder.extract_value(l, 2)

        spot_loc = self.builder.alloca(h_byte, spot_size)  # reserve new env_loc

        args = (env_loc, spot_loc, *args)
        return self.builder.call(func, args)

    def returnn(self, tree):
        value = self.visit(tree.children[0])

        if hasattr(value, "spot_id"):
            print(self.spots.type)
            new_env_loc = self.builder.gep(self.spots, (h(value.spot_id),))  # get env_loc reserved by caller
            env_loc = self.builder.extract_value(value, 1)
            env_loc = self.builder.bitcast(env_loc, new_env_loc.type)

            self.builder.store(self.builder.load(env_loc), new_env_loc)

            new_env_loc = self.builder.bitcast(new_env_loc, h_byte.as_pointer())
            value = self.builder.insert_value(value, new_env_loc, 1)

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
