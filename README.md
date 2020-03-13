# Volpe Programming Language

## Installation

1. You need Python 3.7.0 or higher. [Install here.](https://www.python.org/downloads/)
2. You need pip, the Python package manager. It should come pre-installed with Python.
3. Install the needed packages by running `pip install -r requirements.txt` from the root directory.
4. Optionally add Volpe to path with `python volpe add-path`. This means you can use `volpe file_name.vlp` anywhere (notice the missing `python`).
    - It will update `volpe.bat` with the correct volpe directory and add it to your PATH.
    - You can customise the batch file further if you want to use a specific python installation or environment.

## How to use

Run `volpe --help` to get show the help message. (`-h` for short.)

Write your code in a file ending with `.vlp`. Once you are ready to run it, run `volpe <file_path>`.
`<file_path>` is either a relative path from your working directory or an absolute path.

Use `volpe --verbose` to also see the parsed code. (`-v` for short.)

*If you did not add Volpe to path (step 4 in Installation), prefix all commands with `python`.*

## Documentation

Documentation for Volpe is currently under development. [Link to repository.](https://github.com/ViliamVadocz/Volpe-docs)

## Syntax highlighting

A VSCode compatible syntax highlighter can be found [here.](https://github.com/TheBlocks/VolpeSyntax)

## Community

We have a discord! This is where we post updates, code snippets, and talk about new
ideas for the language.

[Click here to join!](https://discord.gg/ETXcbc6)

## Contribution

Feel free to contribute to the project!

Please run `git update-index --skip-worktree volpe.bat` to ignore changes
to the user-specific `volpe.bat`.
