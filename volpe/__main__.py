"""Volpe Programming Language

Usage:
  volpe -h | --help
  volpe --install
  volpe run <file_path> [-v | --verbose]

Options:
  -h --help     Show this screen.
  --install     Download requirements.
  -v --verbose  Print out the parsed tree and code.
"""


from docopt import docopt


if __name__ == '__main__':
    args = docopt(__doc__)

    # Install requirements.txt
    if args["--install"]:
        from subprocess import check_call
        from sys import executable
        from os import path

        base_path = path.dirname(__file__)
        path_to_requirements = path.abspath(path.join(base_path, "requirements.txt"))
        check_call([executable, "-m", "pip", "install", "-r", path_to_requirements])
        exit()

    # Compile and run volpe code
    if args["run"]:
        import os
        from volpe import run

        file_path = args["<file_path>"]
        assert file_path.split(".")[-1] == "vlp", "Volpe files have the file ending .vlp"
        assert os.path.exists(file_path), f"Could not find file: {file_path}"

        run(file_path, verbose=args["--verbose"])
