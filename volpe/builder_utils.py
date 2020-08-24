from contextlib import contextmanager

from llvmlite import ir

from tree import TypeTree, volpe_assert, get_obj_key_value
from volpe_types import unwrap, is_int, is_flt, is_char


def math(self, tree: TypeTree):
    values = self.visit_children(tree)
    if is_int(tree.return_type) or is_char(tree.return_type):
        return getattr(self, tree.data + "_int")(values)
    if is_flt(tree.return_type):
        return getattr(self, tree.data + "_flt")(values)
    assert False, "can't happen"


def comp(self, tree: TypeTree):
    values = self.visit_children(tree)
    if is_int(tree.children[0].return_type) or is_char(tree.children[0].return_type):
        return getattr(self, tree.data + "_int")(values)
    if is_flt(tree.children[0].return_type):
        return getattr(self, tree.data + "_flt")(values)
    assert False, "can't happen"


@contextmanager
def options(b: ir.IRBuilder, t: ir.Type) -> ir.Value:
    new_block = b.function.append_basic_block("block")
    with b.goto_block(new_block):
        phi_node = b.phi(t)

    def ret(value):
        if not b.block.is_terminated:
            phi_node.add_incoming(value, b.block)
            b.branch(new_block)

    yield ret, phi_node

    b.position_at_end(new_block)


@contextmanager
def build_func(func: ir.Function):
    block = func.append_basic_block("entry")
    builder = ir.IRBuilder(block)

    yield builder, func.args


def mutate_array(self, tree):
    if tree.data == "symbol":
        name = tree.children[0].value

        def fun(value):
            self.local_scope[name] = value

        return self.get_scope(name), fun

    volpe_assert(tree.data == "list_index", "can only index arrays", tree)
    array_value, inner_fun = mutate_array(self, tree.children[0])
    i = self.visit(tree.children[1])

    def fun(value):
        inner_fun(self.builder.insert_element(array_value, value, i))

    return self.builder.extract_element(array_value, i), fun


def assign(self, tree: TypeTree, value):
    if tree.data == "object":
        for i, child in enumerate(tree.children):
            key, attribute = get_obj_key_value(child, i)
            index = list(value.type.type_dict.keys()).index(key)
            assign(self, attribute, self.builder.extract_value(value, index))

    elif tree.data == "attribute":
        obj = tree.children[0].return_type
        index = list(obj.type_dict.keys()).index(tree.children[1])
        value = self.builder.insert_value(self.visit(tree.children[0]), value, index)
        # update scope
        assign(self, tree.children[0], value)

    elif tree.data == "list":
        for i, child in enumerate(tree.children):
            assign(self, child, self.builder.extract_element(value, i))

    elif tree.data == "list_index":
        array_value, fun = mutate_array(self, tree.children[0])
        fun(self.builder.insert_element(array_value, value, self.visit(tree.children[1])))

    else:
        volpe_assert(tree.data == "symbol", f"cannot assign to {tree.data}", tree)
        name = tree.children[0].value
        self.local_scope[name] = value


def build_or_get_function(self, tree):
    closure, args = tree.children[0].return_type, tree.children[1].return_type
    inst = closure.tree.instances[args]
    if not hasattr(inst, "func"):
        arg_type = inst.children[0].return_type
        ret_type = inst.children[1].return_type

        module = self.builder.module
        func_name = str(next(module.func_count))
        func_type = ir.FunctionType(unwrap(ret_type), [unwrap(closure), unwrap(arg_type)])
        inst.func = ir.Function(module, func_type, func_name)

        with build_func(inst.func) as (b, args):
            b: ir.IRBuilder
            with options(b, args[1].type) as (rec, phi):
                rec(args[1])

            new_values = [b.extract_value(args[0], i) for i in range(len(closure.env))]
            env_scope = dict(zip(closure.env.keys(), new_values))
            env_scope["@"] = args[0]

            def scope(name):
                return env_scope[name]

            self.__class__(b, inst.children[1], scope, b.ret, rec, (inst.children[0], phi))

    return inst.func
