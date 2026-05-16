# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Workflow rules

**Branch per feature, no direct pushes to `main`.** Past sessions sometimes committed user-visible changes directly to `main`; from now on:

1. For any user-visible change (new flag, new behavior, perf improvement, output redesign), create a branch named `feat/<short-slug>`, `fix/<short-slug>`, or `perf/<short-slug>`. Trivial doc fixes or CI tweaks can still go straight to `main` if the user asks.
2. Land it via a PR (`gh pr create`) so there's an audit trail. Squash or rebase merge — the project favors a linear history.
3. After merge, add a one- or two-line entry to **`CHANGELOG.md`** under `## [Unreleased]` using the `Added / Changed / Fixed / Removed` buckets. Skip entries that don't affect users (internal refactors, test additions, doc polish).
4. On release: move the `[Unreleased]` items under a new `## [X.Y.Z] — YYYY-MM-DD` heading, bump the version in `Cargo.toml`, tag `vX.Y.Z`, push. The release workflow (`.github/workflows/release.yml`) builds the cross-platform archives automatically.

The current state of the changelog is the source of truth for what's shipped vs. what's queued for the next release — *read it before claiming a feature exists*.

## Project

`gitree` is a Rust CLI that walks a directory tree, finds every git repository under it, and prints a colored tree with each repo's current branch and short HEAD hash. Distributed as a single binary; `libgit2` is statically linked via the `git2` crate (so no system `git` is shelled out).

## Commands

```sh
cargo build --release                  # optimized binary at target/release/gitree
cargo test --release                   # all unit tests (tempfile-backed integration of git_info)
cargo test --release branch_parser     # run a single test by name substring
cargo install --path . --locked        # install into ~/.cargo/bin/gitree
cargo clippy --release -- -D warnings  # lint
```

Running locally during dev:
```sh
target/release/gitree ~/some/dir       # normal scan
target/release/gitree ~ -j 1           # force serial scan (compare against parallel)
target/release/gitree --no-color --no-emoji ~/some/dir  # ASCII / pipe-safe output
```

Releases are tagged `v*` and `.github/workflows/release.yml` builds cross-platform archives. Do not bump the version manually unless cutting a release.

## Architecture

The pipeline is **scan → render → print**, with `git_info` reads happening lazily *during render*, not during scan. Two background concerns — terminal feedback and platform-aware parallelism — wrap that core.

```
main.rs           Args parsing, picks thread count, owns the Spinner, drives the pipeline.
  ├─ scan.rs      Recursive directory walk. Returns Option<Node> (a pruned tree where
  │               every leaf is a repo). Uses rayon::par_iter when parallel=true,
  │               std iter otherwise — no rayon overhead in serial mode.
  ├─ git_info.rs  Per-repo branch / short hash / state via libgit2. Also calls
  │               platform::detect for the icon. Called from render, not scan.
  ├─ platform.rs  Best-guess Rust/Go/Node/Next/Flutter/… from a single read_dir()
  │               on the repo root. No file contents are read.
  ├─ render.rs    Two-line layout per repo (icon+name line, then status line).
  │               Holds the Theme (color/emoji flags) and all paint_* helpers.
  └─ progress.rs  stderr spinner with a Mutex<String> live status. No-op when
                  stderr isn't a TTY. Caller updates status via a callback the
                  scanner invokes on every dir it descends into.
```

### Things that matter when editing

- **Parallelism gating**: `main::pick_thread_count` decides serial vs parallel. Default is rayon-default (all cores), except on Linux where a rotational disk is detected via `/sys/dev/block/<major>:<minor>/.../queue/rotational` — then it falls back to 1. The `-j N` flag overrides. The `parallel: bool` is threaded through `scan::walk` so the serial path can stay `std::iter` (zero rayon cost).

- **Reporter callback is `&(dyn Fn(&Path) + Sync)`**: the scanner calls it on every directory entry, and in parallel mode multiple threads call it concurrently. The Spinner's `Mutex<String>` already handles that.

- **State markers, not dots, for non-normal repos**: `Theme::state_marker` returns `None` for on-branch (normal) and only emits `⚠` / `○` / `✕` for detached / empty / unreadable. The two-line render shows a `└─` continuation under each repo's name; `build_status` decides whether to append `· <hash>` (only OnBranch/Detached have meaningful hashes).

- **Icon slot is 2 cells**: `render::icon_slot` pads narrow glyphs like `▲` (Next.js) so they occupy the same width as wide emoji (🦀, 🐳). Otherwise names misalign across rows.

- **`scan.rs` `should_skip`** prunes `.dotdirs` and a fixed `DEFAULT_SKIP` list (`node_modules`, `target`, `dist`, …) unless `--all` is passed. New skips go in that constant.

- **`git_info::read` returns a `RepoInfo` even on error**: `Unreadable` (open failed), `Empty` (no commits), `Detached`, or `OnBranch`. Render must handle all four — don't `unwrap` on the state.

- **Header / tree color writes go to stdout; spinner writes go to stderr**. Don't swap them: piping to a file should produce clean output with no ANSI control sequences from the spinner.

### Tests

`src/git_info.rs` has a `#[cfg(test)] mod tests` that builds real repos in a `TempDir` via `git2` and exercises all four `RepoState` branches. Tests use `Repository::init_opts(initial_head: "main")` so they don't depend on the host's `init.defaultBranch`. Add new tests there using the same `commit_empty` / `init_with_branch` helpers.

### Adding a new platform icon

1. Add a variant to `Platform` in `src/platform.rs`.
2. Map it in both `icon_emoji` and `icon_ascii`.
3. Insert a detection branch in `detect()` — order matters (more specific markers above more generic ones; e.g. `next.config.*` before `package.json`).
4. Document it in the README's "Platform icons" tables (English + Korean).

## Design notes

`PLAN.md` records the UI redesign decisions (two-line layout, branch-before-hash, `·` separator, no `●` for normal state, narrow-glyph padding). Read it before making layout changes — many of those choices come from real tradeoffs (outlier widths, scan-time outliers like `(no commits)`).
