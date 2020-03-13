"""Volpe Programming Language

Usage:
    volpe -h | --help
    volpe add-path
    volpe <file_path> [-v | --verbose]

Options:
    -h --help     Show this screen.
    -v --verbose  Print out the parsed tree and code.
"""

import os
from sys import version_info
from docopt import docopt


def install():
    needs_restart = False
    path_to_volpe = os.path.abspath(os.path.dirname(os.path.dirname(__file__)))

    # print(path_to_volpe)
    # print("PATH:\n", os.environ["PATH"])
    # print("PATHETX:\n", os.environ["PATHEXT"])

    if path_to_volpe in os.environ["PATH"]:
        print("Volpe already on PATH")

    else:
        # Look up user path in registry.
        import winreg
        reg_key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, r'Environment')
        path = winreg.QueryValueEx(reg_key, 'Path')[0]

        # Add Volpe path to user path.
        print(f"adding {path_to_volpe} to user PATH")
        path_with_volpe = path + os.pathsep + path_to_volpe
        print("setting local user PATH")
        os.system(f'SETX Path "{path_with_volpe}"')
        needs_restart = True

    print("=====")

    if ".PY" in os.environ["PATHEXT"].upper():
        print(".PY already in PATHEXT")

    else:
        # Add .PY to PATHEXT
        print("adding .PY to PATHEXT")
        os.environ["PATHEXT"] += os.pathsep + ".PY"
        pathext_var = os.environ["PATHEXT"]

        print("setting PATHETX")
        os.system(f'SETX PATHEXT "{pathext_var}" /M')
        needs_restart = True
    
    print("=====")

    if needs_restart:
        print("In anything failed, try running this in a console as administrator.")
        print("Please restart this console for changes to take effect.")
    else:
        print("Everything seems good to go!")


def compile_and_run(file_path, verbose=False):
    from run_volpe import run

    assert file_path.split(".")[-1] == "vlp", "Volpe files have the file ending .vlp"
    assert os.path.exists(file_path), f"Could not find file: {file_path}"

    run(file_path, verbose=verbose)


if __name__ == '__main__':
    assert version_info >= (3, 7, 0), "You need Python 3.7 or higher."

    args = docopt(__doc__)

    if args["add-path"]:
        install()
    else:
        compile_and_run(args["<file_path>"], args["--verbose"])
