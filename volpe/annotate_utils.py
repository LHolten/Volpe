from tree import TypeTree
from volpe_types import int1, int64, flt64, char, VolpeList, combine_types


def logic(self, tree: TypeTree):
    ret = self.visit_children(tree)
    assert ret[0] == int1 and ret[1] == int1, "logic operations only work for booleans"
    return int1


def unary_logic(self, tree: TypeTree):
    ret = self.visit_children(tree)[0]
    assert ret == int1, "unary logic operations only work for booleans"
    return int1


def math(self, tree: TypeTree):
    children = self.visit_children(tree)
    ret0 = children[0]
    ret1 = children[1]
    assert combine_types(self, ret0, ret1), "types need to match for math operations"
    if ret0 == int64:
        tree.data = tree.data + "_int"
    elif ret0 == flt64:
        tree.data = tree.data + "_flt"
    elif isinstance(ret0, VolpeList) and tree.data == "add":
        tree.data = "add_list"
    else:
        raise AssertionError("math operations only work for integers and floats")
    return ret0


def unary_math(self, tree: TypeTree):
    ret = self.visit_children(tree)[0]
    if ret == int64:
        tree.data = tree.data + "_int"
    elif ret == flt64:
        tree.data = tree.data + "_flt"
    else:
        raise AssertionError("unary math operations only work for integers and floats")
    return ret


def math_assign(self, tree: TypeTree):
    symbol = tree.children[0]
    symbol.data = "symbol"
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
    assert ret0 == ret1, "types need to match for comparisons"
    if ret0 == int64 or ret0 == char:
        tree.data = tree.data + "_int"
    elif ret0 == flt64:
        tree.data = tree.data + "_flt"
    else:
        raise AssertionError("comparisons only work for integers, floats, and chars")
    return int1






