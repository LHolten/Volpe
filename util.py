from typing import List

import llvmlite.binding as llvm
from lark import Tree
from llvmlite import ir

llvm.initialize()
llvm.initialize_native_target()
llvm.initialize_native_asmprinter()  # yes, even this one

h_bool = ir.IntType(1)
h_int = ir.IntType(32)
h_byte = ir.IntType(8)
target_data = llvm.Target.from_default_triple().create_target_machine().target_data


def h(n):
    return ir.Constant(h_int, n)


class TypeTree(Tree):
    ret = None

    def _pretty_label(self):
        if self.ret is not None:
            return f'{self.data}: {self.ret}'
        return self.data


class Lambda():
    def __init__(self, loc: ir.PointerType, spot_id: int):
        self.loc = loc
        self.spot_id = spot_id


class LambdaAnnotation(ir.LiteralStructType):
    def __init__(self, return_type, args, env: ir.LiteralStructType, env_names: tuple, spots: List):
        fnt = ir.FunctionType(return_type, [h_byte.as_pointer(), h_byte.as_pointer(), *args])  # env offset, spot pointer, args
        super().__init__((fnt.as_pointer(), ir.ArrayType(h_byte, env.get_abi_size(target_data))))
        self.args = args
        self.return_type = return_type
        self.env = env
        self.env_names = env_names
        self.spots = spots
        self.spot_size = 0
        if spots:
            self.spot_size = max(s.get_abi_size(target_data) for s in spots)


# class Environment(ir.LiteralStructType):
#     def __init__(self, env):
#         envs = [e.env for e in env if isinstance(e, LambdaAnnotation)]
#
#         super().__init__((ir.LiteralStructType(env), *envs))
