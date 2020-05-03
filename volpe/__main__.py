"""Volpe Programming Language

Usage:
    volpe -h | --help
    volpe add-path
    volpe <file_path> [-v | --verbose] [-t | --time] [-f | --fast]

Options:
    -h --help     Show this screen.
    -v --verbose  Print out the parsed tree and code.
    -t --time     Shows execution time.
    -f --fast     Use 32 bit instead of 64 bit for floating point numbers.
"""

import os
from sys import version_info
from docopt import docopt


def install():
    # Look up user path in registry.
    import winreg
    reg_key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, r'Environment')
    path = winreg.QueryValueEx(reg_key, "Path")[0]

    path_to_volpe = os.path.abspath(os.path.dirname(__file__))
    path_to_volpe_root = os.path.dirname(path_to_volpe)

    # Update the .bat file
    print("Updating batch file with volpe directory.")
    with open(os.path.join(path_to_volpe_root, "volpe.bat"), "w") as OPATH:
        OPATH.writelines(["@echo off\n", f"python {path_to_volpe} %*"])
    print("=====")

    if path_to_volpe_root in path:
        print("Volpe is already on PATH.")
        
        if not path_to_volpe_root in os.environ["PATH"]:
            print("=====")
            print("Please restart this console for changes to take effect.")

    else:
        # Add Volpe path to user path.
        print(f"Adding {path_to_volpe_root} to user PATH.")
        path_including_volpe = path + os.pathsep + path_to_volpe_root
        print("Setting local user PATH.")
        os.system(f'SETX Path "{path_including_volpe}"')
        
        print("=====")
        print("Please restart this console for changes to take effect.")


def compile_and_run(file_path, verbose=False, show_time=False, fast=False):
    from run_volpe import run

    assert file_path.split(".")[-1] == "vlp", "Volpe files have the file ending .vlp"
    assert os.path.exists(file_path), f"Could not find file: {file_path}"

    run(file_path, verbose=verbose, show_time=show_time, fast=fast)


if __name__ == '__main__':
    assert version_info >= (3, 7, 0), "You need Python 3.7 or higher."

    args = docopt(__doc__)

    if args["add-path"]:
        install()
    else:
        compile_and_run(args["<file_path>"], verbose=args["--verbose"], show_time=args["--time"], fast=args["--fast"])
