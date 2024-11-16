# prog

## Install

```sh
cargo install --git ssh://git@github.com/bytemain/prog.git
# or http
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
  cdtmp   Create a temporary directory
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

then you can use `prog` to manage your repositories.

for example:

```sh
> prog add https://github.com/bytemain/prog
// repository will be clone to ~/0Workspace/github.com/bytemain/prog
```

if you want to change the base directory, you can change the configuration file `~/.prog/config.toml`:

```toml
base = [
    "~/0Workspace"
]
```sh
