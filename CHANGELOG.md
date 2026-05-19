# Changelog

All notable user-visible changes go here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Entry order under each version: **Added · Changed · Fixed · Removed**.
Drop trivial entries (refactors, docs polish, CI tweaks) — this file is for
people running the binary, not contributors.

## [Unreleased]

## [0.2.0] — 2026-05-19

### Added
- Platform icons for ~22 ecosystems, auto-detected from a single `read_dir()`
  on the repo root (Rust, Go, Python, Next.js, Nuxt, Astro, Svelte, Deno,
  Node, Ruby, PHP, Java, Elixir, Flutter, Swift, iOS, Android, Terraform,
  Docker, static web, Makefile).
- Live scanning spinner on stderr showing the currently-visited directory.
  No-op when stderr isn't a TTY.
- `-j, --jobs <N>` flag for parallel directory scan. Defaults to all cores
  except on Linux + HDD (detected via `/sys/block/.../queue/rotational`)
  where it falls back to 1.
- Animated CLI demo SVG embedded at the top of the README.
- "AI install prompt" section in the README for one-paste install via
  Claude Code / Cursor / Codex.

### Changed
- Repo output is now two lines per entry (icon+name, then status) so a
  single outlier branch name doesn't push every hash 10 columns to the
  right. Branch and hash are joined with a tight `·` separator.
- Normal on-branch state no longer prints a `●` — color carries the
  signal. Only `⚠` (detached), `○` (empty), `✕` (unreadable) get a marker.
- `~/personal` scan: ~30ms → ~10ms; `~` scan (~270k dirs): 65s → 7s.

## [0.1.0] — 2026-05-14

Initial release.

### Added
- Tree view of every git repository under a directory.
- Per-repo current branch + 6-char HEAD hash via libgit2 (no shell-out
  to `git`).
- Rounded header box with path, repo count, scan duration.
- `-L, --depth`, `-a, --all`, `--no-color`, `--no-emoji` flags.
- Default-skip list (`node_modules`, `target`, `dist`, …) honored unless
  `--all` is passed.
- Cross-platform release pipeline (Linux x86_64/aarch64,
  macOS x86_64/aarch64, Windows x86_64) on tagged `v*` pushes.

[Unreleased]: https://github.com/rtzr/gitree/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/rtzr/gitree/releases/tag/v0.2.0
[0.1.0]: https://github.com/rtzr/gitree/releases/tag/v0.1.0
