<h1 align="center">🌳 gitree</h1>

<p align="center">
  <em>A tree view of every git repository under a directory,
  with branch and short commit hash at a glance.</em><br/>
  <em>한 디렉토리 아래의 모든 git 리포지토리를 트리로 보여주고,
  현재 브랜치와 6자리 커밋 해시를 한눈에 확인합니다.</em>
</p>

<p align="center">
  <a href="https://github.com/rtzr/gitree/releases/latest"><img src="https://img.shields.io/github/v/release/rtzr/gitree?style=flat-square" alt="Latest release"></a>
  <a href="https://github.com/rtzr/gitree/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/rtzr/gitree/release.yml?style=flat-square" alt="Build status"></a>
  <a href="#license"><img src="https://img.shields.io/github/license/rtzr/gitree?style=flat-square" alt="License"></a>
</p>

<p align="center">
  <a href="#english">English</a> · <a href="#한국어">한국어</a>
</p>

```text
╭─────────────────────────────────────────────────────────╮
│ 🌳 gitree · ~/personal · 26 repos · scanned in 5ms      │
╰─────────────────────────────────────────────────────────╯

personal/
├─ 📦 2msi-landing          ● main           b44f84
├─ 📦 auto-auction          ● main           dde900
├─ 📦 chocoya-location      ○ (no commits)   ------
├─ 📂 work/
│   ├─ 📦 project-x         ● feature/login  c92840
│   └─ 📦 project-y         ⚠ (detached)     112233
└─ 📦 gitree                ● main           88aa14
```

---

<a id="english"></a>

## English

### Why gitree?

If your `~/projects` folder has grown into a forest of clones, `gitree`
gives you a one-shot snapshot: which branch is each repo on, what's the
HEAD commit, and is anything in an unusual state (detached, empty)?
No registration step — point it at a directory and it auto-discovers
every repo underneath.

### Install

**From source** (requires Rust):

```sh
cargo install --git https://github.com/rtzr/gitree
```

**From a release binary** — grab the archive for your platform from the
[Releases page](https://github.com/rtzr/gitree/releases), extract
`gitree`, and put it somewhere on your `PATH`.

Pre-built binaries are published for:

- macOS — Apple Silicon (`aarch64-apple-darwin`) & Intel (`x86_64-apple-darwin`)
- Linux — `x86_64-unknown-linux-gnu` & `aarch64-unknown-linux-gnu`
- Windows — `x86_64-pc-windows-msvc`

### Usage

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

```sh
# Scan your projects folder two levels deep
gitree ~/projects -L 2

# Scan your home directory but include hidden folders
gitree ~ --all

# Plain ASCII output (handy in logs / CI)
gitree ~/projects --no-color --no-emoji
```

### Status legend

| Marker | Meaning                          |
| :----: | -------------------------------- |
|  `●`   | HEAD is on a named branch        |
|  `⚠`   | Detached HEAD                    |
|  `○`   | Repo has no commits yet          |
|  `✕`   | Repo couldn't be opened or read  |

### How it works

`gitree` walks the directory tree from the given root. Whenever it
encounters a directory containing `.git` (folder or file — regular
repos, submodules, and linked worktrees all work), it marks the
directory as a leaf and stops descending. Branches with no repos
underneath are pruned, so the output stays focused on actual project
state.

For each discovered repo, `gitree` reads the current branch label and
the 6-character HEAD commit hash via `libgit2` — no shelling out to
`git`.

---

<a id="한국어"></a>

## 한국어

### 왜 gitree?

`~/projects` 폴더가 클론들로 가득 차 있나요? `gitree`는 한 번 실행으로
**각 리포의 현재 브랜치 · HEAD 커밋 · 비정상 상태(detached/empty 등)**를
스냅샷으로 보여줍니다. 별도 등록 단계 없이 디렉토리를 가리키기만 하면
그 아래의 모든 리포를 자동으로 찾아냅니다.

### 설치

**소스에서 빌드** (Rust 필요):

```sh
cargo install --git https://github.com/rtzr/gitree
```

**미리 빌드된 바이너리** — [Releases](https://github.com/rtzr/gitree/releases)
에서 본인 플랫폼용 아카이브를 받아, 압축을 풀고 `gitree` 실행 파일을
`PATH` 위에 두세요.

지원 플랫폼:

- macOS — Apple Silicon (`aarch64-apple-darwin`) / Intel (`x86_64-apple-darwin`)
- Linux — `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu`
- Windows — `x86_64-pc-windows-msvc`

### 사용법

```text
gitree [경로] [옵션]
```

| 옵션             | 설명                                                                 |
| ---------------- | -------------------------------------------------------------------- |
| `PATH`           | 탐색을 시작할 경로. 생략하면 현재 디렉토리.                          |
| `-L, --depth N`  | 트리 최대 깊이를 N 단계로 제한.                                      |
| `-a, --all`      | 기본 제외 폴더(`node_modules`, `target` 등)도 탐색.                  |
| `--no-color`     | ANSI 컬러 끄기 (파이프로 연결되면 자동으로 꺼집니다).                |
| `--no-emoji`     | 이모지 대신 ASCII 마커(`[D]`, `[G]`, `*`, `!`) 사용.                 |
| `-h, --help`     | 전체 도움말 보기.                                                    |
| `-V, --version`  | 버전 정보 보기.                                                      |

### 예시

```sh
# 프로젝트 폴더를 2단계까지만 스캔
gitree ~/projects -L 2

# 홈 디렉토리를 숨김 폴더 포함해서 스캔
gitree ~ --all

# 컬러/이모지 없이 plain ASCII로 출력 (로그/CI에 유용)
gitree ~/projects --no-color --no-emoji
```

### 상태 마커

| 마커 | 의미                                  |
| :--: | ------------------------------------- |
| `●`  | HEAD가 명시된 브랜치 위에 있음        |
| `⚠`  | HEAD가 분리됨 (detached)              |
| `○`  | 아직 커밋이 없는 리포                 |
| `✕`  | 리포를 열거나 읽지 못함               |

### 동작 원리

`gitree`는 지정한 루트부터 디렉토리 트리를 재귀적으로 워크합니다.
`.git`(폴더 또는 파일 — 일반 리포, 서브모듈, 링크된 워크트리 모두 지원)을
가진 디렉토리를 만나면 **leaf로 처리하고 더 내려가지 않습니다**. 또한
git 리포가 하나도 없는 가지는 출력에서 잘려나가므로(prune), 결과가
실제 프로젝트 상태에만 집중됩니다.

각 리포의 브랜치 라벨과 6자리 단축 해시는 외부 `git` 호출 없이 `libgit2`로
직접 읽습니다.

---

<a id="license"></a>

## License · 라이선스

MIT
