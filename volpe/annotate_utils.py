from tree import TypeTree, volpe_assert, get_obj_key_value
from volpe_types import int1, VolpeObject, VolpeArray, char, is_flt, is_int, is_char


def logic(self, tree: TypeTree):
    ret = self.visit_children(tree)
    volpe_assert(ret[0] == ret[1] == int1, "logic operations only work for booleans", tree)
    return int1


def unary_logic(self, tree: TypeTree):
    ret = self.visit_children(tree)[0]
    volpe_assert(ret == int1, "unary logic operations only work for booleans", tree)
    return int1


def math(self, tree: TypeTree):
    ret = self.visit_children(tree)
    volpe_assert(ret[0] == ret[1], "types need to match for math operations", tree)
    volpe_assert(is_int(ret[0]) or is_flt(ret[0]) or is_char(ret[0]), "can only do math operations with int, flt, or char")
    return ret[0]


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
    ret = self.visit_children(tree)
    volpe_assert(ret[0] == ret[1], "types need to match for comparison operations", tree)
    volpe_assert(ret[0] == char or is_int(ret[0]) or is_flt(ret[0]), "can only compare int, flt and char")
    return int1


def assign(self, scope: dict, tree: TypeTree, value):
    if tree.data == "object":
        volpe_assert(isinstance(value, VolpeObject), "can only destructure object", tree)
        volpe_assert(len(tree.children) == len(value.type_dict), "only full deconstruction is allowed", tree)
        used = set()
        for i, child in enumerate(tree.children):
            key, attribute = get_obj_key_value(child, i)
            volpe_assert(key in value.type_dict, f"object doesn't have attribute {key}", child)
            volpe_assert(key not in used, f"{key} has already been used", child)
            used.add(key)
            assign(self, scope, attribute, value.type_dict[key])

    elif tree.data == "attribute":
        volpe_assert(self.visit(tree) == value, "wrong type in attribute assignment", tree)

    elif tree.data == "list":
        volpe_assert(isinstance(value, VolpeArray), "can only destructure array")
        volpe_assert(value.count == len(tree.children), "array has wrong length")
        for child in tree.children:
            assign(self, scope, child, value.element)

    elif tree.data == "list_index":
        volpe_assert(self.visit(tree) == value, "wrong type in array assignment", tree)

    else:
        volpe_assert(tree.data == "symbol", f"cannot assign to {tree.data}", tree)
        scope[tree.children[0].value] = value

    tree.return_type = value
