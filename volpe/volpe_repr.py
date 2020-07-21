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
    VolpeArray
)

def determine_c_type(volpe_type):
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
        class CObject(Structure):
            _fields_ = [(key, determine_c_type(value)) for key, value in volpe_type.type_dict.items()]
            def __repr__(self):
                res = "{" + ", ".join([key + ": " + str(getattr(self, key)) for (key, _) in self._fields_])
                return res + ",}" if len(self._fields_) == 1 else res + "}"
                    

        return CObject

    if isinstance(volpe_type, VolpeArray):
        element_type = determine_c_type(volpe_type.element)
        length = volpe_type.count

        class CArray(Structure):
            _fields_ = [(f"elem{i}", element_type) for i in range(length)] 

            def __repr__(self):               
                # Special case if elements are chars -> string
                if element_type == c_char:
                    if length == 0:
                        return "\"\""
                    return "\"" + "".join([chr(getattr(self, tup[0])) for tup in self._fields_]) + "\""
                if length == 0:
                    return "[]"
                return "[" + ", ".join([str(getattr(self, tup[0])) for tup in self._fields_]) + "]"

        return CArray

    if isinstance(volpe_type, VolpeClosure):
        class CFunc(Structure):
            _fields_ = [(key, determine_c_type(value)) for key, value in volpe_type.env.items()]
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

    if isinstance(volpe_type, VolpeArray):
        return f"list<{get_type_name(volpe_type.element)}>"
   
    if isinstance(volpe_type, VolpeClosure):
        input_type = remove_obj_brackets(get_type_name(volpe_type.arg))
        return_type = get_type_name(volpe_type.ret)
        return f"func ({input_type}) " + "{" + return_type + "}"

    # Unknown type
    return "?"

# Really hacky solution to removing object brackets for function arguments
def remove_obj_brackets(s):
    if s[-2] == ",":
        return s[1:-2]
    return s[1:-1]
