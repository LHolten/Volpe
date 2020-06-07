from ctypes import (
    c_bool,
    c_int64,
    c_double,
    c_char,
    Structure,
    POINTER
)

from volpe_types import (
    int1,
    int64,
    flt64,
    char,
    VolpeObject,
    VolpeClosure,
    VolpeList
)

def determine_c_type(volpe_type, depth=0):
    """Interpret the volpe type and return a corresponding C type."""
    # Simple types:
    if volpe_type == int1:
        return c_bool
    if volpe_type == int64:
        return c_int64
    if volpe_type == flt64:
        return c_double
    if volpe_type == char:
        return c_char
    
    # Aggregate types:
    if isinstance(volpe_type, VolpeObject):
        elems = volpe_type.type_dict.values()

        class CTuple(Structure):
            _fields_ = [(f"elem{i}", determine_c_type(elem, depth+1)) for i, elem in enumerate(elems)]
            def __repr__(self):
                res = "{" + ", ".join([str(getattr(self, tup[0])) for tup in self._fields_])
                return res + ",}" if len(self._fields_) == 1 else res + "}"
                    

        return POINTER(CTuple) if depth == 0 else CTuple

    if isinstance(volpe_type, VolpeList):
        element_type = determine_c_type(volpe_type.element_type, depth+1)

        class CList(Structure):
            _fields_ = [("elems", POINTER(element_type)), ("length", c_int64)]

            def __repr__(self):
                elems = getattr(self, "elems")
                length = getattr(self, "length")
                
                # Special case if elements are chars -> string
                if element_type == c_char:
                    if length == 0:
                        return "\"\""
                    return "\"" + "".join([chr(elem) for elem in elems[:length]]) + "\""
                if length == 0:
                    return "[]"
                return "[" + ", ".join([str(elem) for elem in elems[:length]]) + "]"

        return POINTER(CList) if depth == 0 else CList

    if isinstance(volpe_type, VolpeClosure):
        class CFunc(Structure):
            _fields_ = [("func", POINTER(None)), ("c_func", POINTER(None)), ("f_func", POINTER(None)), ("env", POINTER(None))]
            def __repr__(self):
                return get_type_name(volpe_type)
        return CFunc
        
    # Unknown type
    return None


def get_type_name(volpe_type):
    """Get short Volpe names for types."""
    # Simple types:
    if volpe_type == int1:
        return "bool"
    if volpe_type == int64:
        return "int64"
    if volpe_type == flt64:
        return "flt64"
    if volpe_type == char:
        return "char"

    # Aggregate types:
    if isinstance(volpe_type, VolpeObject):
        elems = volpe_type.type_dict.values()
        res = "{" + ", ".join([get_type_name(elem) for elem in elems])
        return res + ",}" if len(elems) == 1 else res + "}"

    if isinstance(volpe_type, VolpeList):
        return f"list<{get_type_name(volpe_type.element_type)}>"
   
    if isinstance(volpe_type, VolpeClosure):
        input_type = remove_obj_brackets(get_type_name(volpe_type.arg_type))
        return_type = get_type_name(volpe_type.ret_type)
        return f"func ({input_type}) " + "{" + return_type + "}"

    # Unknown type
    return "?"

# Really hacky solution to removing object brackets for function arguments
def remove_obj_brackets(s):
    if s[-2] == ",":
        return s[1:-2]
    return s[1:-1]
