from typing import Optional
from lark import Tree


class TypeTree(Tree):
    return_type = None

    def _pretty_label(self):
        if self.return_type is not None:
            return f'{self.data}: {self.return_type}'
        return self.data


class VolpeError(Exception):
    def __init__(self, message: str, tree: Optional[TypeTree]=None):
        if tree is not None:
            message += f", line: {tree.meta.line}"
        super().__init__(message)

def volpe_assert(condition: bool, message: str, tree: Optional[TypeTree]=None):
    if not condition:
        raise VolpeError(message, tree)