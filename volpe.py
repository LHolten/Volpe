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


if __name__ == '__main__':
    assert version_info >= (3, 7, 0), "You need Python 3.7 or higher."
    from volpe.__main__ import install, compile_and_run

    args = docopt(__doc__)

    if args["add-path"]:
        install()
    else:
        compile_and_run(args["<file_path>"], args["--verbose"])
