from copy import deepcopy
from typing import Optional, List, Union
from lark import Tree, Token


class TypeTree(Tree):
    return_type = None
    instances = None
    children: List[Union["TypeTree", Token]]

    def _pretty_label(self):
        if self.return_type is not None:
            return f"{self.data}: {self.return_type}"
        return self.data

    def __deepcopy__(self, memo):
        return type(self)(self.data, deepcopy(self.children, memo), self.meta)


def get_obj_key_value(tree: TypeTree, i):
    if tree.data == "item":
        return tree.children[0], tree.children[1]
    return f"_{i}", tree


class VolpeError(Exception):
    def __init__(self, message: str, tree: Optional[TypeTree]=None, stack_trace: Optional[List[TypeTree]]=None):
        if tree is None:
            super().__init__(message)
            return

        # Add type info to error
        types = ", ".join(str(child.return_type) for child in tree.children if isinstance(child, TypeTree))
        message += f"\n  typing: {types}"

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
            for i, line in enumerate(text[first_line - 1 : last_line], first_line):
                padding = " " * (spacing - len(str(i)))
                message += f"\n{padding}{i}| {line.rstrip()}"

        if stack_trace is not None and len(stack_trace) > 0:
            trace = ""
            for bush in stack_trace:
                trace = trace + str(VolpeError(f"{bush.data}", bush)) + "\n"
            message = f"-- stack trace --\n{trace}-- - - - - - - --\n{message}"

        super().__init__(message)


def volpe_assert(condition: bool, message: str, tree: Optional[TypeTree]=None, stack_trace: Optional[List[TypeTree]]=None):
    if not condition:
        raise VolpeError(message, tree, stack_trace)
