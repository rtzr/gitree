# gitree UI 리디자인 계획

## 1. 현재 상황 진단

현재(660767b 기준) `gitree ~/personal` 출력:

```
├─ 📦 2msi-landing
│  ● main          b44f84
├─ 📦 chocoya-location
│  ○ (no commits)  ------
├─ 📦 gitree
│  ● main          660767
├─ 📦 gwansang-android
│  ● master        48ad8d
```

### 1-1. 가독성 문제

| # | 문제 | 근본 원인 |
|---|---|---|
| P1 | `main` 과 해시 사이 8~10칸의 빈 공간 | 형제 리포 사이에서 브랜치 컬럼을 가장 긴 브랜치 ("(no commits)") 폭으로 맞추기 때문. 1개의 outlier가 전체 시각 리듬을 망친다. |
| P2 | 브랜치-해시 사이가 "그냥 공백"이라 둘이 같은 그룹임을 시각적으로 묶어주는 신호가 없다 | 구분 기호/색 변화 부재 |
| P3 | 리포 두 줄 묶음(이름 줄 + 상태 줄)과 다음 리포 사이의 시각적 구분이 약함 — 26개를 스크롤하면 어느 상태가 어느 이름에 속한 건지 가끔 헷갈림 | 두 줄 묶음에 그룹 마커가 없음 (`│`만으로는 약함) |
| P4 | 모든 리포가 동일한 📦 아이콘 — 어떤 게 Rust 프로젝트인지, Android 앱인지, Python 스크립트인지 한눈에 알 수 없음 | 플랫폼 감지 미구현 |
| P5 | 상태 줄이 이름 줄과 동일한 시각 강도라 "이름이 주, 상태가 부"라는 위계감이 없음 | 색/굵기 위계 부재 |

### 1-2. 잘 되어 있는 부분 (유지할 것)

- 둥근 헤더 박스 + 메타 정보
- 색으로 표현되는 상태 (녹/적/황/회)
- 트리 커넥터 (`├─`, `└─`, `│`)
- 스피너 / `--no-color` / `--no-emoji` 폴백
- 6자리 단축 해시 (충분히 짧고 검색 가능)

---

## 2. 디자인 원칙

좋은 터미널 UI에 대한 판단 기준을 먼저 못박는다.

1. **불필요한 공백은 제거** — 정보 밀도가 떨어지면 스캔 속도가 느려진다. 다만 *완전히* 붙이지는 말고, 단위 사이는 식별 가능한 구분자(`·`, `•`)로 분리.
2. **outlier 가 평범한 행을 망치지 않도록** — 컬럼 정렬은 "정보 비교가 유의미할 때만". 브랜치 이름은 비교 대상이 아니므로 column-align 하지 않는다. 해시는 비교 대상이 아니므로 column-align 하지 않는다.
3. **위계는 색의 강도로** — 이름 > 브랜치 > 해시 > 트리선 순으로 시각 강도 감소.
4. **반복되는 prefix 는 dim 처리** — `├─`, `│`, `└─` 같은 트리 글리프는 정보가 아니라 *구조*이므로 회색.
5. **아이콘은 한 자리(1 cell)**. 폭 가변이면 정렬이 깨진다. 이모지는 대부분 2 cell — `UnicodeWidthStr` 로 측정해서 처리.
6. **모든 옵션은 `--no-color`, `--no-emoji` 에서 의미 손실이 없어야** 한다.
7. **속도 회귀 없음** — 플랫폼 감지는 리포 루트의 `read_dir()` 1회로 끝내야 한다 (파일 내용 파싱 금지).

---

## 3. 레이아웃 후보 비교

데이터: 동일한 리포 4개 (Rust, Docker IaC, Node 웹, Python, empty 리포 1개).

### 후보 A — "tight inline" (현재 구조 유지 + 간격만 개선)

```
├─ 🦀 gitree              ● main · 660767b
├─ 🐳 home-server-iac     ● main · a9af34c
├─ 🟢 jellos              ● main · b881d3b
├─ 🐍 saju                ● main · 975968a
└─ 📦 chocoya-location    ○ no commits
```

- 다시 1줄. 이름은 형제끼리 column-align (이름은 비교 대상이라 align 의미 있음).
- 브랜치-해시는 `·` 로 묶고 column-align 안 함 → outlier 영향 0.
- **장점**: 정보 밀도 최고, 26 개도 한 화면.
- **단점**: 사용자가 이전 턴에서 "2 줄이 더 가독성 좋다" 라고 명시했음. 되돌아감.

### 후보 B — "indented status" (2 줄, 들여쓰기 강화 + tight 구분자)

```
├─ 🦀 gitree
│    ╰╴ main · 660767b
├─ 🐳 home-server-iac
│    ╰╴ main · a9af34c
├─ 🟢 jellos
│    ╰╴ main · b881d3b
├─ 🐍 saju
│    ╰╴ main · 975968a
└─ 📦 chocoya-location
     ╰╴ no commits yet
```

- 상태 줄에 `╰╴` (또는 `└╴`, `↳`) 마커를 붙여 "이 줄은 위 리포에 속한다" 라는 시각 단서.
- 브랜치-해시 `·` 로 tight.
- 상태 줄 전체를 dim 으로 한 단계 낮춰서 위계감.
- **장점**: 사용자가 원한 2 줄 구조 유지 + 그룹 단서 + 컬럼 outlier 제거.
- **단점**: 26 줄 → 52 줄. 화면 절반만 보임.

### 후보 C — "compact two-line with separator dot row"

```
├─ 🦀 gitree
│  ● main · 660767b
│
├─ 🐳 home-server-iac
│  ● main · a9af34c
│
├─ 🐍 saju
│  ● main · 975968a
```

- 리포 사이에 빈 `│` 줄을 끼워 호흡감.
- **장점**: 가장 보기 좋음.
- **단점**: 3 줄/리포. 26 개면 78 줄. 너무 길다.

### 후보 B′ — "hash-first, dot separator" (B 변형: 해시 앞)

```
├─ 🦀 gitree
│    ╰╴ 660767b · main
├─ 🐳 home-server-iac
│    ╰╴ a9af34c · main
├─ 📦 chocoya-location
│    ╰╴ ------- · (no commits)
└─ 📦 some-detached
     ╰╴ 112233a · (detached)
```

- 해시 컬럼이 고정폭(6자) 이므로 그 뒤 브랜치 컬럼이 자동 정렬됨 — 코드에서 padding 계산 불필요.
- empty 리포는 `-------` 플레이스홀더로 컬럼 일관성 유지.
- **장점**: 패딩 코드 0, outlier 영향 0, 색 흐름 dim → bright 자연.
- **단점**: 첫 시선이 16진수 → "어떤 브랜치?" 답이 한 박자 늦음 (반대 의견 있음).

### 후보 B″ — "hash-first, separator-less" (B′ 변형: 구두점 제거)

```
├─ 🦀 gitree
│    ╰╴ 660767b  main
├─ 🐳 home-server-iac
│    ╰╴ a9af34c  main
├─ 📦 chocoya-location
│    ╰╴ -------  (no commits)
└─ 📦 some-detached
     ╰╴ 112233a  (detached)
```

- 해시가 고정폭이라 `·` 없이 2칸 공백만으로 경계 명확.
- 구두점 노이즈 최소. 가장 조용한 디자인.
- **장점**: B′ 의 장점 + 구두점 1 글리프 제거.
- **단점**: 색이 없을 때(`--no-color`) 해시-브랜치 구분이 공백 2칸 의존 — 그래도 해시는 항상 16진수 6자라 충분히 구분됨.

### 후보 D — "single line with inline subtle status" (밀도 우선)

```
├─ 🦀 gitree              main · 660767b
├─ 🐳 home-server-iac     main · a9af34c
├─ 🐍 saju                main · 975968a
└─ 📦 chocoya-location    ○ no commits
```

- 후보 A 에서 `●` 점을 *정상 상태에서는 생략*. 비정상(detached/empty/unreadable)에서만 표시.
- 색만으로 상태 인코딩 충분 (녹색 = on branch).
- **장점**: 노이즈 최소. 비정상만 눈에 띈다.
- **단점**: `--no-color` 에서 정상/비정상 구분 약해짐 — 그러면 fallback 으로 `●` 다시 표시.

---

## 4. 추천: 후보 B (2 줄, 브랜치 앞, `·` 구분자)

> 해시 앞 안(B′, B″)을 검토했으나 사용자가 브랜치 앞 안을 선호. 일상적 질문 "어떤 브랜치냐"의 답이 먼저 시야에 들어와야 한다는 판단.

### 이유

- 사용자의 이전 명시적 선호 (2 줄) 존중.
- 그룹 마커 `╰╴` 로 P3 해결.
- 브랜치-해시 `·` tight 로 P1, P2 해결 (column-align 제거 → outlier 영향 0).
- 상태 줄 dim 으로 P5 해결.
- 플랫폼 아이콘 (5절) 으로 P4 해결.
- 시선 흐름: 브랜치(bold green) → `·`(dim) → 해시(yellow dim) — 정보 우선순위와 시각 강도 일치.

### 최종 mockup (색은 ANSI 코드명으로 표시)

```
personal/
├─ 🦀 gitree                     (이름: bold)
│    ╰╴ main · 660767b           (마커/`·`/해시: bright_black 또는 dim, 브랜치: green bold)
├─ 🐳 home-server-iac
│    ╰╴ main · a9af34c
├─ 🟢 jellos
│    ╰╴ main · b881d3b
├─ 🐍 saju
│    ╰╴ main · 975968a
├─ 📦 chocoya-location
│    ╰╴ no commits yet           (전체 yellow dim)
└─ 📦 some-broken
     ╰╴ (detached) · 112233a     (브랜치: red bold, 해시: red dim)
```

### 색 사양 (`owo-colors` 기준)

| 요소 | 색 | 굵기 |
|---|---|---|
| 이름 (정상) | default | bold |
| 이름 (empty) | default | bold + dimmed |
| 트리 커넥터 (`├─`, `│`, `└─`) | bright_black | normal |
| 상태 마커 (`╰╴`) | bright_black | normal |
| 브랜치 — on-branch | green | normal |
| 브랜치 — detached | red | normal |
| 브랜치 — empty / unreadable | yellow / bright_black | dimmed |
| `·` 구분자 | bright_black | normal |
| 해시 | yellow | dimmed |
| 비정상 dot `⚠ ○ ✕` | 상태색 | bold (detached만), normal (그 외) |

> dot `●` 자체는 후보 D 의 아이디어를 일부 차용해서 **정상 상태에서는 생략** 한다.
> 이유: 정상 표시가 26 번 반복되면 노이즈. 색만으로 충분히 표현된다.
> 비정상(`⚠`, `○`, `✕`) 만 마커 자리에 표시.

---

## 5. 플랫폼 / 프레임워크 아이콘 매핑

### 5-1. 감지 전략

- 리포 루트의 `read_dir()` 1회 호출 → 파일/디렉터리 이름 set 생성.
- 파일 내용은 읽지 않는다 (속도, 의존성 회피).
- 우선순위 리스트 위에서 아래로 매칭. 첫 매치를 채택.
- 매치 없으면 fallback `📦`.

### 5-2. 매핑 테이블 (우선순위 위 → 아래)

| 우선순위 | 마커 파일/디렉토리 | 플랫폼 | 이모지 | ASCII 폴백 |
|---|---|---|---|---|
| 1 | `pubspec.yaml` | Flutter / Dart | 🐦 | `[FL]` |
| 2 | `Package.swift` | Swift | 🦅 | `[SW]` |
| 3 | `*.xcodeproj` 디렉토리 | iOS / Xcode | 🍎 | `[iOS]` |
| 4 | `build.gradle.kts` + `app/` | Android | 🤖 | `[AN]` |
| 5 | `Cargo.toml` | Rust | 🦀 | `[RS]` |
| 6 | `go.mod` | Go | 🐹 | `[GO]` |
| 7 | `mix.exs` | Elixir | 💧 | `[EX]` |
| 8 | `Gemfile` | Ruby | 💎 | `[RB]` |
| 9 | `composer.json` | PHP | 🐘 | `[PHP]` |
| 10 | `pom.xml` | Java (Maven) | ☕ | `[JV]` |
| 11 | `pyproject.toml` / `requirements.txt` / `setup.py` / `Pipfile` | Python | 🐍 | `[PY]` |
| 12 | `deno.json` / `deno.jsonc` | Deno | 🦕 | `[DN]` |
| 13 | `package.json` | Node.js (일반) | 🟢 | `[JS]` |
| 14 | `index.html` (only, w/o package.json) | Static web | 🌐 | `[WEB]` |
| 15 | `*.tf` / `*.hcl` | Terraform/IaC | 🌍 | `[TF]` |
| 16 | `Dockerfile` (only) | Docker | 🐳 | `[DK]` |
| 17 | `Makefile` (only) | C/C++/기타 | 🔧 | `[MK]` |
| 18 | (default) | unknown | 📦 | `[G]` |

> `*.xcodeproj` / `*.tf` / `*.hcl` 만 glob 매칭 필요. 나머지는 단순 string equality. glob 도 `ends_with(".xcodeproj")` 로 충분 → 정규식 의존성 추가 안 함.

### 5-3. Node.js 세분화 (선택 / v2 검토)

`package.json` 의 dependencies 를 *읽으면* Next/Nuxt/Astro/Svelte 등을 더 정확히 구분 가능. 하지만:
- 파일 내용 파싱 = `serde_json` 의존성 추가
- 26 개 리포 × JSON 파싱 → 속도 영향
- 디렉토리 entry 만으로 추정: `next.config.*` → Next, `nuxt.config.*` → Nuxt, `astro.config.*` → Astro, `svelte.config.*` → Svelte. 이것만 추가하면 의존성 0 으로 80% 케이스 커버 가능.

→ **v1 결정**: `next.config.*`, `nuxt.config.*`, `astro.config.*`, `svelte.config.*` 만 추가 매칭. 위 테이블의 13 (`package.json`) 위에 13a~13d 로 끼워넣음.

| 우선순위 추가 | 마커 (디렉토리 entry 의 prefix) | 플랫폼 | 이모지 |
|---|---|---|---|
| 13a | `next.config.` | Next.js | ▲ |
| 13b | `nuxt.config.` | Nuxt | 💚 |
| 13c | `astro.config.` | Astro | 🚀 |
| 13d | `svelte.config.` | Svelte | 🧡 |

> `▲` (U+25B2) 는 폭 1, Next.js 의 공식 로고 모티프와 일치하고 다른 이모지보다 시각적으로 가벼움. iTerm/WezTerm/Ghostty 등 현대 터미널에서 모두 잘 렌더된다.

### 5-4. 아이콘 폭 처리

이모지는 대부분 폭 2 (`UnicodeWidthStr::width` 가 2 반환). `▲` 는 폭 1. 정렬할 때 *플랫폼 아이콘이 들어간 라벨의 폭*을 직접 측정해서 처리하면 됨 — 현재 코드의 `name_w` 알고리즘과 동일 로직 적용.

문제: `▲` 와 `🦀` 가 같은 리스트에 섞이면 한 줄은 폭 1, 다른 줄은 폭 2 → 다음에 이어지는 이름 시작 위치가 한 칸씩 어긋남. 

**해결**: 모든 아이콘 뒤에 1 공백 + 측정된 폭만큼 추가 패딩.
구체적으로는 폭 2 짜리 (대부분 이모지) 뒤 1 칸, 폭 1 짜리 (`▲`) 뒤 2 칸을 두면 시각적으로 동일 정렬.

```rust
fn icon_padding(icon: &str) -> &'static str {
    if UnicodeWidthStr::width(icon) >= 2 { " " } else { "  " }
}
```

---

## 6. 구현 계획

### 6-1. 새 파일 / 변경 파일

| 파일 | 변경 종류 | 설명 |
|---|---|---|
| `src/platform.rs` | 신규 | `detect(repo_root: &Path) -> Platform` 함수. `Platform` enum 과 `icon()`/`ascii_icon()`. |
| `src/git_info.rs` | 변경 | `RepoInfo` 에 `platform: Platform` 추가. `read()` 가 `platform::detect()` 호출. |
| `src/render.rs` | 변경 | 후보 B 레이아웃으로 변경. dot 자동 생략 로직. `·` 구분자. 상태 줄 dim. |
| `src/main.rs` | 변경 없음 (또는 모듈 등록 1줄) | |
| `README.md` | 변경 | 새 출력 예시 + 플랫폼 아이콘 범례 추가 (한/영 양쪽). |

### 6-2. `Platform` enum 설계

```rust
pub enum Platform {
    Rust, Go, Node, NextJs, NuxtJs, Astro, Svelte, Deno,
    Python, Ruby, Php, Java, Elixir,
    Flutter, Swift, IOSXcode, Android,
    Terraform, Docker, Makefile, StaticWeb,
    Unknown,
}

impl Platform {
    fn icon_emoji(self) -> &'static str { ... }
    fn icon_ascii(self) -> &'static str { ... }
    fn label(self) -> &'static str { ... } // 도움말/legend 용
}
```

### 6-3. `platform::detect` 알고리즘

```rust
pub fn detect(root: &Path) -> Platform {
    let Ok(entries) = std::fs::read_dir(root) else { return Platform::Unknown; };
    let names: HashSet<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();

    // 우선순위대로 체크
    if names.contains("pubspec.yaml") { return Platform::Flutter; }
    if names.contains("Package.swift") { return Platform::Swift; }
    if names.iter().any(|n| n.ends_with(".xcodeproj")) { return Platform::IOSXcode; }
    if names.contains("build.gradle.kts") && names.contains("app") { return Platform::Android; }
    if names.contains("Cargo.toml") { return Platform::Rust; }
    // ... 나머지
    Platform::Unknown
}
```

### 6-4. 렌더링 흐름

```rust
// repo 한 개의 출력 (Dir 내부에서 호출):
//   line 1: {prefix}{connector}{icon} {name}
//   line 2: {prefix}{continuation}{marker} {branch_glyph?}{branch} · {hash}
//
// 정상 상태면 branch_glyph 생략, 비정상이면 마커 자리에 ⚠ ○ ✕
//
// continuation = "│  " (not last) 또는 "   " (last)  ← 폭 동일
// marker       = "╰╴ " (또는 ASCII fallback "`- ")
```

### 6-5. ASCII 폴백 (`--no-emoji`)

- 아이콘 → 위 테이블 ASCII 칼럼 (`[RS]`, `[FL]`, ...)
- 상태 dot → `*` `!` `o` `?` (현행 유지)
- 마커 `╰╴` → `\`- ` (백슬래시 + 하이픈 + 공백) 또는 `+- `
- 구분자 `·` → ` - ` (공백 하이픈 공백) — `·` 가 ASCII 아니지만 한 byte 가 아닐 뿐 BMP. `--no-color` 와 `--no-emoji` 는 별개. emoji 옵션에서만 ASCII 화.

### 6-6. 단계별 작업

1. `Platform` enum + `detect()` 작성 (단위 테스트: `tempdir` 에 marker 파일 만들어서 분기 검증).
2. `RepoInfo` 에 `platform` 추가, `git_info::read()` 에서 채움.
3. `render.rs` 의 `colored_label`, `render_children` 수정 — 아이콘 출처가 `theme.repo_icon()` → `info.platform.icon(theme)` 로 바뀜.
4. 상태 줄에 `╰╴` 마커, `·` 구분자, 정상상태 dot 생략, 전체 dim.
5. 컬럼 align 제거 (branch_w 계산 코드 삭제).
6. 로컬에서 `gitree ~/personal` 으로 확인 (스크린샷 또는 텍스트 캡처).
7. README 갱신 — 새 예시 출력 + 플랫폼 범례 표.
8. commit / push / `cargo install --path .`

---

## 7. 확정된 결정 사항

1. **레이아웃**: 후보 B (2 줄, 브랜치 앞, `·` 구분자).
2. **정상 상태 dot**: 생략. 색만으로 표현. `--no-color` 에서는 비정상만 `⚠ ○ ✕` 표시되므로 정상은 자연스럽게 마커 없음.
3. **Node 세분화**: 채택. `next.config.*` → ▲, `nuxt.config.*` → 💚, `astro.config.*` → 🚀, `svelte.config.*` → 🧡 추가.
4. **마커 글리프**: `└─` (트리 커넥터와 동일 글리프로 일관성).
5. **구분자 글리프**: `·` (U+00B7).
6. **이모지**: 5-2 표 그대로.

---

## 8. 비-목표 (이 작업에서 다루지 않음)

- working tree dirty/clean 표시
- ahead/behind upstream
- stash 개수
- 동시 스캔 (rayon 등)
- 헤더 박스 재디자인
- `--no-color` 환경에서 색 외의 위계 표현 추가

위는 향후 별도 작업으로 다룬다.
