from tree import TypeTree, volpe_assert, get_obj_key_value
from volpe_types import is_int, is_flt, is_char


def math(self, tree: TypeTree):
    values = self.visit_children(tree)
    if is_int(tree.return_type):
        return getattr(self, tree.data + "_int")(values)
    if is_char(tree.return_type):
        # Use unsigned division and modulus for chars
        if tree.data in ["div", "mod"]: 
            return getattr(self, tree.data + "_uint")(values)
        return getattr(self, tree.data + "_int")(values)
    if is_flt(tree.return_type):
        return getattr(self, tree.data + "_flt")(values)
    assert False, "can't happen"


def comp(self, tree: TypeTree):
    values = self.visit_children(tree)
    if is_int(tree.children[0].return_type) or is_char(tree.children[0].return_type):
        return getattr(self, tree.data + "_int")(values)
    if is_flt(tree.children[0].return_type):
        return getattr(self, tree.data + "_flt")(values)
    assert False, "can't happen"


def assign(self, tree: TypeTree, value):
    if tree.data == "object":
        for i, child in enumerate(tree.children):
            key, attribute = get_obj_key_value(child, i)
            index = list(value.type.type_dict.keys()).index(key)
            assign(self, attribute, self.builder.extract_value(value, index))

    elif tree.data == "attribute":
        obj = tree.children[0].return_type
        index = list(obj.type_dict.keys()).index(tree.children[1])
        value = self.builder.insert_value(self.visit(tree.children[0]), value, index)
        # update scope
        assign(self, tree.children[0], value)

    elif tree.data == "array":
        for i, child in enumerate(tree.children):
            assign(self, child, self.builder.extract_element(value, i))

    elif tree.data == "array_index":
        array, index = self.visit_children(tree)
        new_array = self.builder.insert_element(self.visit(tree.children[0]), value, index)
        assign(self, tree.children[0], new_array)

    else:
        volpe_assert(tree.data == "symbol", f"cannot assign to {tree.data}", tree)
        name = tree.children[0].value
        self.local_scope[name] = value
