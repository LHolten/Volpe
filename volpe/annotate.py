from typing import Callable, List, Optional

from lark.visitors import Interpreter
from lark import Token

from annotate_utils import logic, unary_logic, unary_math, math_assign, comp, chain_comp, assign
from c_interop import VolpeCFunc
from tree import TypeTree, volpe_assert, get_obj_key_value
from volpe_types import int64, flt64, char, VolpeObject, VolpeClosure, VolpeArray, int1, VolpePointer, unknown, \
    is_pointer, is_flt, is_char, is_int
from version_dependent import is_ascii
from tree import VolpeError


class AnnotateScope(Interpreter):
    def __init__(self, tree: TypeTree, scope: Callable, args=None, stack_trace: Optional[List[TypeTree]]=None):
        self.scope = scope
        self.local_scope = dict()
        self.used = set()
        self.stack_trace: List[TypeTree] = [] if stack_trace is None else stack_trace

        if args is not None:
            assign(self, self.local_scope, args[0], args[1])

        value = self.visit(tree.children[1])
        assign(self, self.local_scope, tree.children[0], value)
        if len(tree.children) == 3:
            tree.return_type = self.visit(tree.children[2])
        else:
            tree.return_type = VolpeObject(dict())

    def assert_(self, condition: bool, message: str, tree: Optional[TypeTree]=None):
        volpe_assert(condition, message, tree, self.stack_trace)

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

    def object(self, tree: TypeTree):
        scope = dict()
        for i, child in enumerate(tree.children):
            key, attribute = get_obj_key_value(child, i)
            self.assert_(key not in scope, f"attribute names have to be unique, `{key}` is not", tree)
            scope[key] = self.visit(attribute)
        return VolpeObject(scope)

    def attribute(self, tree: TypeTree):
        obj, key = self.visit(tree.children[0]), tree.children[1]
        self.assert_(isinstance(obj, VolpeObject), "only objects have attributes", tree)
        self.assert_(key in obj.type_dict, f"this object does not have an attribute named {key}", tree)
        return obj.type_dict[tree.children[1]]

    def func(self, tree: TypeTree):
        tree.children = [TypeTree("inst", tree.children, tree.meta)]
        tree.instances = dict()
        return VolpeClosure(tree=tree, scope=self.get_scope())

    def func_call(self, tree: TypeTree):
        self.stack_trace.append(tree)
        closure, args = self.visit_children(tree)
        ret = closure.ret_type(self, args)
        self.stack_trace.pop()
        return ret

    def assign(self, tree: TypeTree):
        AnnotateScope(tree, self.get_scope(), stack_trace=self.stack_trace)
        return tree.return_type

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
            raise VolpeError(err.msg, tree, self.stack_trace)
        self.assert_(is_ascii(text), "strings can only have ascii characters", tree)

        tree.children = []
        for eval_character in text:
            tree.children.append(TypeTree("character", [Token("CHARACTER", "'" + eval_character + "'")], tree.meta))
        self.visit_children(tree)
        return VolpeArray(char, len(tree.children))

    def array_index(self, tree: TypeTree):
        volpe_array, index = self.visit_children(tree)
        self.assert_(isinstance(volpe_array, VolpeArray), "can only index arrays", tree)
        self.assert_(index == int64, "can only index with an integer", tree)
        return volpe_array.element

    def array_size(self, tree: TypeTree):
        volpe_array = self.visit_children(tree)[0]
        self.assert_(isinstance(volpe_array, VolpeArray), "can only get size of arrays", tree)
        return int64

    def array(self, tree: TypeTree):
        self.assert_(len(tree.children) > 0, "array needs at least one value", tree)
        element_type = self.visit(tree.children[0])
        for child in tree.children[1:]:
            self.assert_(element_type == self.visit(child), "different types in array", tree)
        return VolpeArray(element_type, len(tree.children))

    def constant_array(self, tree: TypeTree):
        element_type = self.visit(tree.children[0])
        return VolpeArray(element_type, int(tree.children[1].value))

    def constant_array_like(self, tree: TypeTree):
        tree.data = "constant_array"
        element_type, parent_array = self.visit_children(tree)
        self.assert_(isinstance(parent_array, VolpeArray), "can only get size of arrays", tree)
        return VolpeArray(element_type, parent_array.count)

    def convert_int(self, tree: TypeTree):
        self.assert_(self.visit(tree.children[0]) == int64, "can only convert int", tree)
        return flt64

    def convert_flt(self, tree: TypeTree):
        self.assert_(self.visit(tree.children[0]) == flt64, "can only convert float", tree)
        return int64

    def if_then(self, tree: TypeTree):
        tree.data = "implication"
        tree.children[1] = TypeTree("return_n", [tree.children[1]], tree.meta)
        return self.visit(tree)

    def c_import(self, tree: TypeTree):
        from os import path
        import clang.cindex
        index = clang.cindex.Index.create()

        directory = path.dirname(path.abspath(tree.meta.file_path))
        import_path = path.join(directory, *[child.value for child in tree.children[:-1]]) + ".h"

        options = clang.cindex.TranslationUnit.PARSE_INCOMPLETE + \
            clang.cindex.TranslationUnit.PARSE_SKIP_FUNCTION_BODIES
        res: clang.cindex.Cursor = index.parse(import_path, options=options).cursor
        name = tree.children[-1].value

        for child in res.get_children():
            child: clang.cindex.Cursor
            child_type: clang.cindex.Type = child.type.get_canonical()
            if child_type.kind != clang.cindex.TypeKind.FUNCTIONPROTO:
                continue
            if child.mangled_name != name:
                continue
            return VolpeCFunc(child_type, name)
        self.assert_(False, f"could not find {name} in {import_path}")

    def make_pointer(self, tree: TypeTree):
        array = self.visit(tree.children[0])
        self.assert_(isinstance(array, VolpeArray), "can only make pointer from array", tree)
        return VolpePointer(array.element)

    def math(self, tree: TypeTree):
        ret = self.visit_children(tree)
        self.assert_(ret[0] == ret[1], "types need to match for math operations", tree)
        self.assert_(is_int(ret[0]) or is_flt(ret[0]) or is_char(ret[0]),
                     "can only do math operations with int, flt, or char", tree)
        return ret[0]

    # Boolean logic
    implication = logic
    logic_and = logic
    logic_or = logic
    logic_not = unary_logic

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
