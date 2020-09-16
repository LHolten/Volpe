# Volpe Programming Language

## Installation

1. Clone the master branch of this repository with `git clone https://github.com/LHolten/Volpe.git`.
2. You need Python 3.6.0 or higher, although we recommend at least 3.7.0. [Install here.](https://www.python.org/downloads/)
3. You need pip, the Python package manager. It should come pre-installed with Python.
4. Install the required Python packages by running `pip install -r requirements.txt`.
5. Optionally add Volpe to path with `python volpe add-path`. This means you can use `volpe file_name.vlp` anywhere.
    - It will create a `bin` folder and put either `volpe.bat` (windows) or `volpe` (linux) inside.
    - The new `bin` folder will be added to PATH and the batch/bash script will have the correct volpe directory.
    - You can customise the batch/bash file further if you want to use a specific python installation or environment.

## How to use

Run `volpe --help` to show a help message. (`-h` for short.)

Write your code in a file ending with `.vlp`. Once you are ready to run it, run `volpe <file_path>`.
`<file_path>` is either a relative path from your working directory or an absolute path.

*If you did not add Volpe to path (step 5 in Installation), prefix all commands with `python`.*

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
