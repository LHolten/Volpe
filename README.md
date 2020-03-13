# Volpe Programming Language

## Installation

1. You need Python 3.7.0 or higher. [Install here.](https://www.python.org/downloads/)
2. You need pip, the Python package manager. It should come pre-installed with Python.
3. Install the needed packages by running `pip install -r requirements.txt` from the root directory.
4. Optionally add Volpe to path with `python volpe add-path`. This means you can use `volpe file_name.vlp` anywhere (notice the missing `python`).
    - Note that your default behaviour when clicking on a python file has to be to run it.
    - Set the default behaviour to open with interpreter.

## How to use

Run `volpe --help` to get show the help message. (`-h` for short.)

Write your code in a file ending with `.vlp`. Once you are ready to run it, run `volpe <file_path>`.
`<file_path>` is either a relative path from your working directory or an absolute path.

Use `volpe --verbose` to also see the parsed code. (`-v` for short.)

*If you did not add Volpe to path (step 4 in Installation), prefix all commands with `python`.*

If adding to path automatically did not work for you, add the the project directory to path manually and add `.PY` to PATHEXT.

## Documentation

Documentation for Volpe is currently under development. [Link to repository.](https://github.com/ViliamVadocz/Volpe-docs)

## Syntax highlighting

A VSCode compatible syntax highlighter can be found [here.](https://github.com/TheBlocks/VolpeSyntax)
