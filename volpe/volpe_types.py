from typing import Dict, Callable

import llvmlite.binding as llvm
from llvmlite import ir

from tree import TypeTree

llvm.initialize()
llvm.initialize_native_target()
llvm.initialize_native_asmprinter()  # yes, even this one

int1 = ir.IntType(1)
# int32 = ir.IntType(32)
int64 = ir.IntType(64)
int8 = ir.IntType(8)
pint8 = int8.as_pointer()
# flt32 = ir.FloatType()
flt64 = ir.DoubleType()
char = ir.IntType(8)
unknown = ir.VoidType()
copy_func = ir.FunctionType(pint8, [pint8])
free_func = ir.FunctionType(unknown, [pint8])
unknown_func = ir.FunctionType(unknown, [pint8, pint8])

target_data = llvm.Target.from_default_triple().create_target_machine().target_data


class VolpeObject(ir.LiteralStructType):
    def __init__(self, type_dict: Dict[str, ir.Type]):
        super().__init__(type_dict.values())
        self.type_dict = type_dict

    def set(self, name: str, t: ir.Type):
        self.type_dict[name] = t
        self.__init__(self.type_dict)


class VolpeList(ir.LiteralStructType):
    def __init__(self, element_type: ir.Type):
        super().__init__([element_type.as_pointer(), int64])
        self.element_type = element_type


class VolpeBlock:
    def __init__(self, tree: TypeTree, instance_scope: Callable):
        self.arg_object = tree.children[0]
        self.block = tree.children[1]
        self.tree = tree
        self.outside_used = set()

        self.frozen_scope = instance_scope()

    def call(self, arg_type, closure, annotate):
        args = tuple_assign(dict(), self.arg_object, arg_type)
        args["@"] = closure

        def get_instance_scope():
            def scope(name):
                if name in args.keys():
                    return args[name]
                self.outside_used.add(name)
                return self.frozen_scope(name)

            return scope

        annotate(self.block, get_instance_scope, closure.ret(annotate, arg_type))
        self.block.return_type = closure.func.return_type

    def update(self, closure):
        self.tree.return_type = closure


class VolpeClosure(ir.LiteralStructType):
    def __init__(self, block):
        super().__init__([unknown_func.as_pointer(), copy_func.as_pointer(), free_func.as_pointer(), pint8])
        self.func = unknown_func
        self.blocks = [block]
        self.checked = False

    def call(self, arg_type, annotate):
        if self.checked:  # we have already been here
            combine_types(annotate, self.func.args[1], arg_type)
            return self.func.return_type
        self.checked = True

        for block in self.blocks:
            block.call(arg_type, self, annotate)

        return self.func.return_type

    def ret(self, annotate, arg_type):
        def ret(value_type):
            if self.func.return_type is not unknown:
                combine_types(annotate, value_type, self.func.return_type)
            self.update(ir.FunctionType(value_type, [pint8, arg_type]))
        return ret

    def update(self, func: ir.FunctionType):
        super().__init__([func.as_pointer(), copy_func.as_pointer(), free_func.as_pointer(), pint8])
        self.func = func


def combine_types(annotate, t1, *t):
    if isinstance(t1, VolpeClosure):
        for t2 in t:
            if t1.checked:
                t2.call(t1.func.args[1], annotate)
                combine_types(annotate, t1.func.return_type, t2.func.return_type)
            elif t2.checked:
                t1.call(t2.func.args[1], annotate)
                combine_types(annotate, t1.func.return_type, t2.func.return_type)
            t1.blocks.extend(t2.blocks)
            for block in t1.blocks:
                block.update(t1)
    if isinstance(t1, VolpeObject):
        for t2 in t:
            assert len(t1.type_dict.values()) == len(t2.type_dict.values()), "object has different size"
            for k in t1.type_dict.keys():
                combine_types(annotate, t1.type_dict[k], t2.type_dict[k])
    if isinstance(t1, VolpeList):
        for t2 in t:
            combine_types(annotate, t1.element_type, t2.element_type)
    else:
        assert all(t1 == t2 for t2 in t), "all elements should have the same type"
    return t1


def tuple_assign(scope: Dict, tree: TypeTree, value_type):
    if tree is None:
        return scope
    if tree.data == "object":
        assert isinstance(value_type, VolpeObject), "can only destructure objects"
        assert len(tree.children) == len(value_type.type_dict.values())

        for i, child in enumerate(tree.children):
            tuple_assign(scope, child, value_type.type_dict[f"_{i}"])
    else:
        assert tree.data == "symbol"
        scope[tree.children[0].value] = value_type

    return scope
