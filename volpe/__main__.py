"""Volpe Programming Language

Usage:
  volpe -h | --help
  volpe <file_path> [-v | --verbose]

Options:
  -h --help     Show this screen.
  -v --verbose  Print out the parsed tree and code.
"""

import os
from sys import version_info
from docopt import docopt


if __name__ == '__main__':
    assert version_info >= (3, 7, 0), "You need Python 3.7 or higher."

    args = docopt(__doc__)

    # Compile and run volpe code
    from volpe import run

    file_path = args["<file_path>"]
    assert file_path.split(".")[-1] == "vlp", "Volpe files have the file ending .vlp"
    assert os.path.exists(file_path), f"Could not find file: {file_path}"

    run(file_path, verbose=args["--verbose"])
