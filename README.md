# prog

## Installation

```sh
cargo install --git https://github.com/bytemain/prog.git
```

## Usage

```sh
> prog --help
Usage: prog [COMMAND]

Commands:
  add     Add a new repository
  find    Find a repository by keyword
  sync    Sync repositories
  shell   Activate shell
  import  Import repositories from a path
  remove  Remove a repository by path
  clean   Clean up repositories
  list    List all repositories
  init    Initialize configuration
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

You can use `prog` to manage your repositories.

For example:

```sh
> prog add https://github.com/bytemain/prog
# The repository will be cloned to ~/0Workspace/github.com/bytemain/prog
```

To change the base directory, modify the `base` field in the configuration file `~/.prog/config.toml`:

```toml
base = [
    "~/0Workspace"
]
```

Then you can find the repository by keyword:

```sh
> prog find prog
# This will list all repositories that contain the keyword "prog"
```

If there are many results for the keyword, you will be prompted to select one.

You can also list all repositories:

```sh
> prog list
```

## Shell Integration

Add the following code to your shell configuration file (e.g., `~/.bashrc`, `~/.zshrc`):

```sh
eval "$(prog shell zsh)"
```

Then you can use `p` to change to the repository directory without needing to paste the path:

```sh
# From any directory
> p vscode
# This will change the directory to ~/0Workspace/github.com/microsoft/vscode
```

## Debug

use `PROG_LOG="debug"` to enable debug logs