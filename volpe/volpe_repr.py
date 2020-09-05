from ctypes import (
    sizeof,
    c_bool,
    c_uint8,
    c_int64,
    c_double,
    c_char,
    Structure,
)

from volpe_types import int1, int64, flt64, char, VolpeObject, VolpeClosure, VolpeArray, unwrap
from target import target_data

ENCODING = "ascii"
DEBUG_BUFFER = False


def get_repr(val):
    if hasattr(val, "value"):
        val = val.value
    if hasattr(val, "decode"):
        try:
            val = val.decode(ENCODING)
        except UnicodeDecodeError:
            # not valid ascii, just use default repr (but remove the b)
            return repr(val)[1:]
    return repr(val)


def determine_c_type(volpe_type):
    """Interpret the volpe type and return a corresponding C type."""
    if DEBUG_BUFFER:
        return get_padding(64)

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
        # Create fields with padding in between to counteract abi incompatibility.
        # Fields begin with '*' to avoid name collision with Structure attributes.
        fields = []
        pos = 0

        for i, (key, value) in enumerate(volpe_type.type_dict.items()):
            ll_type = unwrap(volpe_type)._get_ll_pointer_type(target_data).element_type
            pad_needed = target_data.get_element_offset(ll_type, i) - pos
            if pad_needed != 0:
                fields.append((f"*pad_at_{pos}_size_{pad_needed}", get_padding(pad_needed)))
                pos += pad_needed

            c_type = determine_c_type(value)
            fields.append((f"*{key}", c_type))
            pos += sizeof(c_type)

        class CObject(Structure):
            _fields_ = fields

            def __repr__(self):
                # Filter out padding fields.
                keys = [key for (key, _) in self._fields_ if key[:4] != "*pad"]
                # Field names are being shown only if they don't begin with an underscore.
                res = "{" + ", ".join(
                    [("" if key[1] == "_" else f"{key[1:]}: ") + get_repr(getattr(self, key)) for key in keys]
                ) + "}"
                return res

        return CObject

    if isinstance(volpe_type, VolpeArray):

        class CArray(Structure):
            _fields_ = [("elements", determine_c_type(volpe_type.element) * volpe_type.count)]

            def __repr__(self):
                if volpe_type.element == char:
                    try:
                        return f'"{bytes(self).decode(ENCODING)}"'
                    except UnicodeDecodeError:
                        # if not valid ascii, use default repr, but replace b' ' with b" "
                        return f'"{repr(self.elements[:])[2:-1]}"'
                else:
                    return repr(self.elements[:])

        return CArray

    if isinstance(volpe_type, VolpeClosure):

        class CFunc(Structure):
            _fields_ = [(key, determine_c_type(value)) for key, value in volpe_type.env.items()]

            def __repr__(self):
                return f"closure (line {volpe_type.tree.meta.line})"

        return CFunc

    # Unknown type
    return None


def get_padding(size):
    class Buffer(Structure):
        _fields_ = [("data", c_uint8 * size)]

        # repr for testing (hexdump)
        def __repr__(self):
            nibbles = []
            for byte in self.data:
                nibbles.append((byte & 0b11110000) >> 4)
                nibbles.append(byte & 0b00001111)
            res = "hexdump:\n"
            for i, nibble in enumerate(nibbles):
                res += hex(nibble)[2:]
                if i % 2 == 1:
                    res += " "
                if i % 16 == 15:
                    res += "\n"
            return res

    return Buffer
