from tree import TypeTree, volpe_assert
from unification_copy import var
from volpe_types import int1, VolpeObject, VolpeArray, int64


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
    tree.children[1] = TypeTree(operation, [symbol, expression], tree.meta)

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
            obj_scope[f"_{i}"] = shape(self, scope, child)
        tree.return_type = VolpeObject(obj_scope)

    elif tree.data == "list":
        element_type = var()
        for child in tree.children:
            volpe_assert(self.unify(element_type, shape(self, scope, child)), "different types in list", tree)
        tree.return_type = VolpeArray(element_type, len(tree.children))

    elif tree.data == "list_index":
        volpe_list = shape(self, scope, tree.children[0])
        index = self.visit(tree.children[1])
        tree.return_type = var()
        volpe_assert(self.unify(volpe_list, VolpeArray(tree.return_type)), "can only mutate arrays", tree)
        volpe_assert(self.unify(index, int64), "can only index with an integer", tree)

    else:
        assert tree.data == "symbol"  # no message?
        tree.return_type = scope[tree.children[0].value] = var()
    return tree.return_type
