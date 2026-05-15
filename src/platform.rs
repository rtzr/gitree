use std::collections::HashSet;
use std::path::Path;

/// Best-guess primary platform / framework for a repo, inferred by looking
/// at the names of entries in the repo root (no file content is read).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Rust,
    Go,
    Python,
    NextJs,
    NuxtJs,
    Astro,
    Svelte,
    Deno,
    Node,
    Ruby,
    Php,
    Java,
    Elixir,
    Flutter,
    Swift,
    Ios,
    Android,
    Terraform,
    Docker,
    Makefile,
    StaticWeb,
    Unknown,
}

impl Platform {
    /// Wide-glyph (usually emoji) icon. Most are width 2, `▲` is width 1.
    pub fn icon_emoji(self) -> &'static str {
        match self {
            Platform::Rust => "🦀",
            Platform::Go => "🐹",
            Platform::Python => "🐍",
            Platform::NextJs => "▲",
            Platform::NuxtJs => "💚",
            Platform::Astro => "🚀",
            Platform::Svelte => "🧡",
            Platform::Deno => "🦕",
            Platform::Node => "🟢",
            Platform::Ruby => "💎",
            Platform::Php => "🐘",
            Platform::Java => "☕",
            Platform::Elixir => "💧",
            Platform::Flutter => "🐦",
            Platform::Swift => "🦅",
            Platform::Ios => "🍎",
            Platform::Android => "🤖",
            Platform::Terraform => "🌍",
            Platform::Docker => "🐳",
            Platform::Makefile => "🔧",
            Platform::StaticWeb => "🌐",
            Platform::Unknown => "📦",
        }
    }

    pub fn icon_ascii(self) -> &'static str {
        match self {
            Platform::Rust => "[RS]",
            Platform::Go => "[GO]",
            Platform::Python => "[PY]",
            Platform::NextJs => "[NX]",
            Platform::NuxtJs => "[NU]",
            Platform::Astro => "[AS]",
            Platform::Svelte => "[SV]",
            Platform::Deno => "[DN]",
            Platform::Node => "[JS]",
            Platform::Ruby => "[RB]",
            Platform::Php => "[PHP]",
            Platform::Java => "[JV]",
            Platform::Elixir => "[EX]",
            Platform::Flutter => "[FL]",
            Platform::Swift => "[SW]",
            Platform::Ios => "[iOS]",
            Platform::Android => "[AN]",
            Platform::Terraform => "[TF]",
            Platform::Docker => "[DK]",
            Platform::Makefile => "[MK]",
            Platform::StaticWeb => "[WEB]",
            Platform::Unknown => "[G]",
        }
    }
}

pub fn detect(root: &Path) -> Platform {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Platform::Unknown;
    };
    let names: HashSet<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();

    // Helper: does any entry name start with the given prefix?
    let any_starts = |prefix: &str| names.iter().any(|n| n.starts_with(prefix));
    let any_ends = |suffix: &str| names.iter().any(|n| n.ends_with(suffix));

    // Order matters: higher-specificity markers first.
    if names.contains("pubspec.yaml") {
        return Platform::Flutter;
    }
    if names.contains("Package.swift") {
        return Platform::Swift;
    }
    if any_ends(".xcodeproj") || any_ends(".xcworkspace") {
        return Platform::Ios;
    }
    // Android: gradle + an `app/` subdirectory is the standard project layout.
    if (names.contains("build.gradle.kts") || names.contains("build.gradle"))
        && names.contains("app")
    {
        return Platform::Android;
    }
    if names.contains("Cargo.toml") {
        return Platform::Rust;
    }
    if names.contains("go.mod") {
        return Platform::Go;
    }
    if names.contains("mix.exs") {
        return Platform::Elixir;
    }
    if names.contains("Gemfile") {
        return Platform::Ruby;
    }
    if names.contains("composer.json") {
        return Platform::Php;
    }
    if names.contains("pom.xml") || names.contains("build.gradle") {
        return Platform::Java;
    }
    if names.contains("pyproject.toml")
        || names.contains("requirements.txt")
        || names.contains("setup.py")
        || names.contains("Pipfile")
    {
        return Platform::Python;
    }
    if names.contains("deno.json") || names.contains("deno.jsonc") {
        return Platform::Deno;
    }
    // Node frameworks before generic Node — only the config-file presence is
    // checked, no JSON parsing.
    if any_starts("next.config.") {
        return Platform::NextJs;
    }
    if any_starts("nuxt.config.") {
        return Platform::NuxtJs;
    }
    if any_starts("astro.config.") {
        return Platform::Astro;
    }
    if any_starts("svelte.config.") {
        return Platform::Svelte;
    }
    if names.contains("package.json") {
        return Platform::Node;
    }
    if any_ends(".tf") || any_ends(".hcl") {
        return Platform::Terraform;
    }
    if names.contains("Dockerfile") || any_starts("docker-compose") {
        return Platform::Docker;
    }
    if names.contains("index.html") {
        return Platform::StaticWeb;
    }
    if names.contains("Makefile") {
        return Platform::Makefile;
    }
    Platform::Unknown
}
