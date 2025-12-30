# prog

## Installation

```sh
cargo install --git https://github.com/bytemain/prog.git
```

## Usage

You can use `prog` to manage your repositories, it can clone repositories according to the path rules.

For example:

```sh
> prog add https://github.com/bytemain/prog
```

The repository will be cloned to `~/0Workspace/github.com/bytemain/prog`.

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

### Bash/Zsh

Add the following code to your shell configuration file (e.g., `~/.bashrc`, `~/.zshrc`):

```sh
eval "$(prog shell bash)"
# or for zsh:
eval "$(prog shell zsh)"
```

### PowerShell

Add the following code to your PowerShell profile (e.g., `$PROFILE`):

```powershell
Invoke-Expression (prog shell powershell | Out-String)
```

**Note:** Using `Invoke-Expression "$(prog shell powershell)"` (with quotes around `$()`) will not work due to PowerShell's command substitution behavior. Use `Out-String` as shown above.

### Using the integration

Then you can use `p` to change to the repository directory without needing to paste the path:

```sh
# From any directory
> p vscode
# This will change the directory to ~/0Workspace/github.com/microsoft/vscode
```

## Debug

use `PROG_LOG="debug"` to enable debug logs
