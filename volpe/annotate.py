from typing import Callable
from sys import version_info

from lark.visitors import Interpreter
from lark import Token
from copy import deepcopy

from annotate_utils import logic, unary_logic, math, unary_math, math_assign, comp, chain_comp, assign
from tree import TypeTree, volpe_assert, VolpeError, get_obj_key_value
from volpe_types import int64, flt64, char, VolpeObject, VolpeClosure, VolpeArray, int1
from version_dependent import is_ascii


class AnnotateScope(Interpreter):
    def __init__(self, tree: TypeTree, scope: Callable, args=None):
        self.scope = scope
        self.local_scope = dict()
        self.used = set()

        if args is not None:
            assign(self, self.local_scope, args[0], args[1])

        tree.children[-1] = TypeTree("return_n", [tree.children[-1]], tree.meta)

        def ret(value_type):
            if tree.return_type is None:
                tree.return_type = value_type
            volpe_assert(tree.return_type == value_type, "block has different return types", tree)

        self.ret = ret

        self.visit_children(tree)  # sets tree.return_type

    def visit(self, tree: TypeTree):
        tree.return_type = getattr(self, tree.data)(tree)
        if tree.return_type is None:
            tree.return_type = int1
        return tree.return_type

    def get_scope(self):
        local_scope = self.local_scope.copy()

        def scope(name, tree: TypeTree):
            if name in local_scope:
                return local_scope[name]
            return self.scope(name, tree)

        return scope

    def symbol(self, tree: TypeTree):
        return self.get_scope()(tree.children[0].value, tree)

    def block(self, tree: TypeTree):
        AnnotateScope(tree, self.get_scope())
        return tree.return_type

    def object(self, tree: TypeTree):
        scope = dict()
        for i, child in enumerate(tree.children):
            key, attribute = get_obj_key_value(child, i)
            volpe_assert(key not in scope, f"attribute names have to be unique, `{key}` is not", tree)
            scope[key] = self.visit(attribute)
        return VolpeObject(scope)

    def attribute(self, tree: TypeTree):
        obj, key = self.visit(tree.children[0]), tree.children[1]
        volpe_assert(isinstance(obj, VolpeObject), "only objects have attributes", tree)
        volpe_assert(key in obj.type_dict, f"this object does not have an attribute named {key}", tree)
        return obj.type_dict[tree.children[1]]

    def func(self, tree: TypeTree):
        tree.children = [TypeTree("inst", tree.children, tree.meta)]
        tree.instances = dict()
        return VolpeClosure(tree=tree, scope=self.get_scope())

    def func_call(self, tree: TypeTree):
        closure, args = self.visit_children(tree)

        if args not in closure.tree.instances:
            new_tree = closure.tree.instances[args] = deepcopy(closure.tree.children[0])
            closure.tree.children.append(new_tree)

            closure.env = dict()

            def scope(name, t_tree: TypeTree):
                if name == "@":
                    return closure
                closure.env[name] = closure.scope(name, t_tree)
                return closure.env[name]

            AnnotateScope(new_tree.children[1], scope, (new_tree.children[0], args))

        return closure.tree.instances[args].children[1].return_type

    def return_n(self, tree: TypeTree):
        self.ret(self.visit(tree.children[0]))

    def assign(self, tree: TypeTree):
        value = self.visit(tree.children[1])
        assign(self, self.local_scope, tree.children[0], value)

    @staticmethod
    def integer(_: TypeTree):
        return int64

    @staticmethod
    def character(_: TypeTree):
        return char

    @staticmethod
    def escaped_character(_: TypeTree):
        return char

    def string(self, tree: TypeTree):
        tree.data = "array"
        # Evaluate string using Python
        try:
            text = eval(tree.children[0])
        except SyntaxError as err:
            raise VolpeError(err.msg, tree)
        volpe_assert(is_ascii(text), "strings can only have ascii characters", tree)
        volpe_assert(len(text) > 0, "empty strings are not allowed", tree)

        tree.children = []
        for eval_character in text:
            tree.children.append(TypeTree("character", [Token("CHARACTER", "'" + eval_character + "'")], tree.meta))
        self.visit_children(tree)
        return VolpeArray(char, len(tree.children))

    def array_index(self, tree: TypeTree):
        volpe_array, index = self.visit_children(tree)
        volpe_assert(isinstance(volpe_array, VolpeArray), "can only index arrays", tree)
        volpe_assert(index == int64, "can only index with an integer", tree)
        return volpe_array.element

    def array_size(self, tree: TypeTree):
        volpe_array = self.visit_children(tree)[0]
        volpe_assert(isinstance(volpe_array, VolpeArray), "can only get size of arrays", tree)
        return int64

    def array(self, tree: TypeTree):
        volpe_assert(len(tree.children) > 0, "array needs at least one value", tree)
        element_type = self.visit(tree.children[0])
        for child in tree.children[1:]:
            volpe_assert(element_type == self.visit(child), "different types in array", tree)
        return VolpeArray(element_type, len(tree.children))

    def constant_array(self, tree: TypeTree):
        element_type = self.visit(tree.children[0])
        return VolpeArray(element_type, int(tree.children[1].value))

    def constant_array_like(self, tree: TypeTree):
        tree.data = "constant_array"
        element_type, parent_array = self.visit_children(tree)
        volpe_assert(isinstance(parent_array, VolpeArray), "can only get size of arrays", tree)
        return VolpeArray(element_type, parent_array.count)

    def convert_int(self, tree: TypeTree):
        volpe_assert(self.visit(tree.children[0]) == int64, "can only convert int", tree)
        return flt64

    def convert_flt(self, tree: TypeTree):
        volpe_assert(self.visit(tree.children[0]) == flt64, "can only convert float", tree)
        return int64

    def if_then(self, tree: TypeTree):
        tree.data = "implication"
        tree.children[1] = TypeTree("return_n", [tree.children[1]], tree.meta)
        return self.visit(tree)

    # Boolean logic
    implication = logic
    logic_and = logic
    logic_or = logic
    logic_not = unary_logic

    # Mathematics
    add = math
    mod = math
    mul = math
    sub = math
    div = math
    # power = math
    negate = unary_math
    add_assign = math_assign
    sub_assign = math_assign
    mul_assign = math_assign
    div_assign = math_assign
    mod_assign = math_assign

    # Comparison
    chain_comp = chain_comp
    equals = comp
    not_equals = comp
    greater = comp
    less = comp
    greater_equals = comp
    less_equals = comp

    def __default__(self, tree: TypeTree):
        raise NotImplementedError("annotate", tree.data)
