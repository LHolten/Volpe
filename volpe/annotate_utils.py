from unification import var

from tree import TypeTree, volpe_assert
from volpe_types import int1, VolpeObject


def logic(self, tree: TypeTree):
    ret = self.visit_children(tree)
    volpe_assert(
        self.unify(ret[0], int1) and self.unify(ret[1], int1),
        "logic operations only work for booleans",
        tree
    )
    return int1


def unary_logic(self, tree: TypeTree):
    ret = self.visit_children(tree)[0]
    volpe_assert(self.unify(ret, int1), "unary logic operations only work for booleans", tree)
    return int1


def math(self, tree: TypeTree):
    children = self.visit_children(tree)
    ret0 = children[0]
    ret1 = children[1]
    volpe_assert(self.unify(ret0, ret1), "types need to match for math operations", tree)
    return ret0


def unary_math(self, tree: TypeTree):
    return self.visit_children(tree)[0]


def math_assign(self, tree: TypeTree):
    symbol = tree.children[0]
    expression = tree.children[1]

    # Make the current node an assign.
    operation = tree.data.replace("_assign", "")
    tree.data = "assign"

    # Make a child node with the math operation.
    tree.children[1] = TypeTree(operation, [symbol, expression])

    return self.visit(tree)


def comp(self, tree: TypeTree):
    children = self.visit_children(tree)
    ret0 = children[0]
    ret1 = children[1]
    volpe_assert(self.unify(ret0, ret1), "types need to match for comparison operations", tree)
    return int1


def shape(self, scope: dict, tree: TypeTree):
    if tree.data == "object":
        obj_scope = dict()
        for i, child in enumerate(tree.children):
            name = f"_{i}"
            obj_scope[name] = shape(self, scope, child)
        tree.return_type = VolpeObject(obj_scope)
        return tree.return_type

    if tree.data == "list_index":
        self.visit(tree)
        return tree.return_type

    assert tree.data == "symbol"  # no message?
    tree.return_type = var()
    scope[tree.children[0].value] = tree.return_type
    return tree.return_type
