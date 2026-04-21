# prog

## Installation

```sh
cargo install --git https://github.com/bytemain/prog.git
```


## Setup

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


## Usage

You can use `p` to manage your repositories, it can clone repositories according to the path rules.

For example:

```sh
> p add https://github.com/bytemain/prog
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
> p find prog
# This will list all repositories that contain the keyword "prog"
```

If there are many results for the keyword, you will be prompted to select one.

You can also list all repositories:

```sh
> p list
```

## Check for unsynced changes

Before switching machines, you can check if any tracked repository has uncommitted or unpushed changes:

```sh
> p check
```

The output groups repositories by issue:

- **Dirty** — uncommitted changes (modified, untracked, or conflicted files)
- **Unpushed** — local commits ahead of the upstream branch
- **No upstream** — local branch has no upstream configured
- **Detached HEAD** — current HEAD is detached
- **Unreadable** — `git status` failed (e.g. corrupted repo)

The exit code is non-zero when any issue is found, so it can be used in shell hooks.

Useful flags:

- `-d`, `--dirty-only` — only report repositories with uncommitted changes
- `--json` — print machine-readable JSON output

## Debug

use `PROG_LOG="debug"` to enable debug logs
