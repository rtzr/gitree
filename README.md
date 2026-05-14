# gitree

Tree view of git repositories under a directory, with current branch and
short commit hash. Point it at your projects folder (or your home) and
see every repo's state in one glance.

```text
╭─────────────────────────────────────────────────────────╮
│ 🌳 gitree · ~/personal · 26 repos · scanned in 5ms      │
╰─────────────────────────────────────────────────────────╯

personal/
├─ 📦 2msi-landing          ● main           b44f84
├─ 📦 auto-auction          ● main           dde900
├─ 📦 best-driver           ● main           84fd07
├─ 📦 chocoya-location      ○ (no commits)   ------
├─ 📂 work/
│   ├─ 📦 project-x         ● feature/login  c92840
│   └─ 📦 project-y         ⚠ (detached)     112233
└─ 📦 gitree                ● main           88aa14
```

## Install

### From source (any platform with Rust)

```sh
cargo install --git https://github.com/rtzr/gitree
```

### From release binaries

Pre-built binaries for Linux, macOS, and Windows are attached to each
[GitHub Release](https://github.com/rtzr/gitree/releases). Download the
archive for your platform, extract `gitree`, and put it on your `PATH`.

## Usage

```text
gitree [PATH] [OPTIONS]
```

| Flag             | Description                                                       |
| ---------------- | ----------------------------------------------------------------- |
| `PATH`           | Root path to scan. Defaults to the current directory.             |
| `-L, --depth N`  | Limit the tree to N levels deep.                                  |
| `-a, --all`      | Don't skip default-ignored dirs (`node_modules`, `target`, ...).  |
| `--no-color`     | Disable ANSI colors (also auto-disabled when piping).             |
| `--no-emoji`     | Use ASCII markers (`[D]`, `[G]`, `*`, `!`) instead of emoji.      |
| `-h, --help`     | Show full help.                                                   |
| `-V, --version`  | Show version.                                                     |

### Examples

Scan your projects folder two levels deep:

```sh
gitree ~/projects -L 2
```

Scan your home directory but include hidden folders:

```sh
gitree ~ --all
```

Use plain ASCII output (handy in logs):

```sh
gitree ~/projects --no-color --no-emoji
```

## Status legend

| Marker | Meaning                                            |
| ------ | -------------------------------------------------- |
| `●`    | HEAD is on a named branch                          |
| `⚠`    | Detached HEAD                                      |
| `○`    | Repo has no commits yet                            |
| `✕`    | Repo couldn't be opened or read                    |

## How it works

`gitree` walks the directory tree from the given root. Whenever it finds
a directory that contains a `.git` (folder or file — both regular repos
and submodules / linked worktrees are supported), it treats that
directory as a leaf and stops descending. Branches with no repos under
them are pruned so the output stays focused on actual project state.

For each discovered repo, `gitree` reads the current branch label and
the 6-character HEAD commit hash via `libgit2` — there's no shelling
out to `git`.

## License

MIT
