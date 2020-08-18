from copy import deepcopy
from typing import Optional
from lark import Tree


class TypeTree(Tree):
    return_type = None
    instances = None

    def _pretty_label(self):
        if self.return_type is not None:
            return f'{self.data}: {self.return_type}'
        return self.data

    def __deepcopy__(self, memo):
        return type(self)(self.data, deepcopy(self.children, memo), self.meta)


def get_obj_key_value(tree: TypeTree, i):
    if len(tree.children) == 2:
        return tree.children[0], tree.children[1]
    return f"_{i}", tree.children[0]


class VolpeError(Exception):
    def __init__(self, message: str, tree: Optional[TypeTree] = None):
        if tree is None:
            super().__init__(message)
            return

        if not hasattr(tree.meta, "file_path"):
            # file_path in tree.meta has not been initialized
            super().__init__(message + f", line: {tree.meta.line}")
            return

        # Pretty error printing that shows the code block
        first_line = tree.meta.line
        last_line = tree.meta.end_line
        spacing = len(str(last_line))

        with open(tree.meta.file_path, "r") as f:
            text = f.readlines()    
            for i, line in enumerate(text[first_line-1: last_line], first_line):
                padding = " " * (spacing - len(str(i)))
                message += f"\n{padding}{i}| {line.rstrip()}"

        super().__init__(message)


def volpe_assert(condition: bool, message: str, tree: Optional[TypeTree] = None):
    if not condition:
        raise VolpeError(message, tree)
