from llvmlite import ir

from builder_utils import build_func, free, options
from volpe_types import int64, char, VolpeArray, VolpeObject, size, int32, pint8

string_type = VolpeArray(char)
string_list = VolpeArray(string_type)
string_obj = VolpeObject({"_0": string_list})


def build_main(module, run_func, printf_func):
    main_func = ir.Function(module, ir.FunctionType(int32, [int32, char.as_pointer().as_pointer()]), "main")
    with build_func(main_func) as (b, args):
        num_args = b.zext(args[0], int64)
        closure = b.call(run_func, [])
        func = b.extract_value(closure, 0)
        env_ptr = b.extract_value(closure, 3)

        pointer = b.call(module.malloc, [b.mul(num_args, size(string_type))])
        pointer = b.bitcast(pointer, string_type.unwrap().as_pointer())

        with options(b, int64) as (ret, phi):
            ret(int64(0))

        with b.if_then(b.icmp_unsigned("<", phi, num_args)):
            string = b.load(b.gep(args[1], [phi]))
            new_string = string_to_volpe(b, string)
            b.store(new_string, b.gep(pointer, [phi]))

            ret(b.add(phi, int64(1)))

        arguments = string_list.unwrap()(ir.Undefined)
        arguments = b.insert_value(arguments, pointer, 0)
        arguments = b.insert_value(arguments, num_args, 1)

        arg_obj = string_obj.unwrap()(ir.Undefined)
        arg_obj = b.insert_value(arg_obj, arguments, 0)

        res = b.call(func, [env_ptr, arg_obj])
        free(b, closure)

        print_obj = VolpeObject({"_0": string_type}).unwrap()(ir.Undefined)
        print_obj = b.insert_value(print_obj, res, 0)
        b.call(b.extract_value(printf_func, 0), [pint8(ir.Undefined), print_obj])

        b.ret(int32(0))


def string_to_volpe(b: ir.IRBuilder, string: ir.Value):
    with options(b, int64) as (ret, phi):
        ret(int64(0))

    character = b.load(b.gep(string, [phi]))
    with b.if_then(b.icmp_unsigned("!=", character, char(0))):
        ret(b.add(phi, int64(1)))

    new_string = string_type.unwrap()(ir.Undefined)
    new_string = b.insert_value(new_string, string, 0)
    new_string = b.insert_value(new_string, phi, 1)

    return new_string
