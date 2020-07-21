from ctypes import (
    c_bool,
    c_int64,
    c_double,
    c_char,
    Structure,
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

ENCODING = "ascii"

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
        class c_char_wrapper(c_char):
            # Use Python byte repr but cut off the `b` preffix.
            def __repr__(self):
                return repr(bytes(self))[1:]
        return c_char_wrapper
    
    # Aggregate types:
    if isinstance(volpe_type, VolpeObject):
        class CObject(Structure):
            _fields_ = [(key, determine_c_type(value)) for key, value in volpe_type.type_dict.items()]

            def __repr__(self):
                # Field names are being shown only if they don't begin with an underscore.
                res = "{" + ", ".join(
                    [
                        ("" if key[0] == "_" else f"{key}: ") + str(getattr(self, key)) 
                        for (key, _) in self._fields_
                    ]
                )
                res += ",}" if len(self._fields_) == 1 else "}"
                return res

        return CObject

    if isinstance(volpe_type, VolpeArray):
        class CArray(Structure):
            _fields_ = [("elements", determine_c_type(volpe_type.element) * volpe_type.count)]
            def __repr__(self):
                if volpe_type.element == char:
                    return f"\"{bytes(self).decode(ENCODING)}\""
                else:
                    return repr(self.elements[:])

        return CArray

    if isinstance(volpe_type, VolpeClosure):
        class CFunc(Structure):
            _fields_ = [(key, determine_c_type(value)) for key, value in volpe_type.env.items()]

            def __repr__(self):
                return "func"
        return CFunc
        
    # Unknown type
    return None
