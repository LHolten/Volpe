"""Volpe Programming Language

Usage:
    volpe -h | --help
    volpe add-path
    volpe <file_path> [-v | --verbose] [-t | --time] [-c | --console]

Options:
    -h --help     Show this screen.
    -v --verbose  Print out the parsed tree and code.
    -t --time     Shows execution time.
    -c --console  Compile to object file.
"""

import os
from sys import version_info, platform
from docopt import docopt


def install():
    path_to_volpe = os.path.abspath(os.path.dirname(__file__))
    path_to_volpe_root = os.path.dirname(path_to_volpe)
    path_to_volpe_bin = os.path.join(path_to_volpe_root, "bin")

    batch_file_path = os.path.join(path_to_volpe_bin, "volpe.bat")
    bash_file_path = os.path.join(path_to_volpe_bin, "volpe")

    if not os.path.isdir(path_to_volpe_bin):
        os.makedirs(path_to_volpe_bin)

    if platform == "win32" and not os.path.isfile(batch_file_path):
        # Update the batch file.
        print("Creating batch file with volpe directory.")
        with open(batch_file_path, "w") as batch_file:
            batch_file.writelines(["@echo off\n", f"python {path_to_volpe} %*"])

    elif platform == "linux" and not os.path.isfile(bash_file_path):
        # Update the bash file.
        print("Creating bash file with volpe directory.")
        with open(bash_file_path, "w") as bash_file:
            bash_file.writelines(["#!/bin/bash\n", f"python {path_to_volpe} $@"])

        # Add executable permissions.
        print("Adding executable permission to bash file with chmod")
        import stat
        st = os.stat(bash_file_path)
        os.chmod(bash_file_path, st.st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

    if path_to_volpe_bin in os.environ["PATH"]:
        print("Volpe is already on PATH.")
        return

    if platform == "win32":
        # Make sure it is not on PATH before trying to add it
        import winreg
        reg_key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment")
        path = winreg.QueryValueEx(reg_key, "Path")[0]

        if path_to_volpe_bin not in path:
            with open("PATH_BACKUP.txt", "w") as f:
                print("Writing current PATH to PATH_BACKUP.txt in case it gets accidentally truncated.")
                f.write(path)
            # Add Volpe path to user path.
            print(f"Adding {path_to_volpe_bin} to user PATH.")
            path_including_volpe = path + os.pathsep + path_to_volpe_bin
            os.system(f'SETX Path "{path_including_volpe}"')

        print("Please restart this console for changes to take effect.")

    elif platform == "linux":
        # Add Volpe path to profile.
        home = os.path.expanduser("~")
        profile = os.path.join(home, ".profile")
        command = f"export PATH=\"$PATH:{path_to_volpe_bin}\""

        with open(profile, "r") as profile_file:
            contents = profile_file.read()
            already_there = command in contents
            needs_newline = contents[-1] == "\n"

        if not already_there:
            print(f"Adding {command} to {profile}")
            with open(profile, "a") as profile_file:
                if needs_newline:
                    profile_file.write("\n")
                profile_file.write(f"{command}\n")

        print("Please reboot for changes to take effect.")

    else:
        print(f"Unsupported platform: {platform}\nPlease go tell the developers to fix this or make a contribution.")


def run(file_path, verbose=False, show_time=False, console=False):
    import traceback
    from parse import parse_trees, volpe_llvm
    from compile import compile_and_run
    from tree import VolpeError

    assert file_path.split(".")[-1] == "vlp", "Volpe files have the file ending .vlp"
    assert os.path.exists(file_path), f"Could not find file: {file_path}"

    try:
        tree = parse_trees(file_path, dict())
        llvm = volpe_llvm(tree, verbose, verbose, console)
    except VolpeError as err:
        if verbose:
            traceback.print_exc()
        else:
            print(err)
        return

    compile_and_run(llvm, tree.return_type, verbose, show_time, console)


if __name__ == "__main__":
    assert version_info >= (3, 6, 0), "You need Python 3.6 or higher."

    args = docopt(__doc__)

    if args["add-path"]:
        install()
    else:
        run(args["<file_path>"], args["--verbose"], args["--time"], args["--console"])
