use std::io::IsTerminal;
use std::path::Path;
use std::time::Duration;

use owo_colors::OwoColorize;
use unicode_width::UnicodeWidthStr;

use crate::git_info::{self, RepoInfo, RepoState};
use crate::platform::Platform;
use crate::scan::{Node, NodeKind};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub color: bool,
    pub emoji: bool,
}

impl Theme {
    pub fn resolve(no_color: bool, no_emoji: bool) -> Self {
        let color = !no_color && std::io::stdout().is_terminal();
        Self {
            color,
            emoji: !no_emoji,
        }
    }

    fn dir_icon(self) -> &'static str {
        if self.emoji { "📂" } else { "[D]" }
    }

    fn header_icon(self) -> &'static str {
        if self.emoji { "🌳" } else { "gitree" }
    }

    fn platform_icon(self, p: Platform) -> &'static str {
        if self.emoji {
            p.icon_emoji()
        } else {
            p.icon_ascii()
        }
    }

    fn state_marker(self, state: RepoState) -> Option<&'static str> {
        // On-branch is the common case — the eye finds the green branch text
        // immediately, so a leading dot just adds noise. Only non-normal states
        // get a marker.
        if matches!(state, RepoState::OnBranch) {
            return None;
        }
        Some(if !self.emoji {
            match state {
                RepoState::OnBranch => unreachable!(),
                RepoState::Detached => "!",
                RepoState::Empty => "o",
                RepoState::Unreadable => "?",
            }
        } else {
            match state {
                RepoState::OnBranch => unreachable!(),
                RepoState::Detached => "⚠",
                RepoState::Empty => "○",
                RepoState::Unreadable => "✕",
            }
        })
    }
}

pub fn count_repos(node: &Node) -> usize {
    match &node.kind {
        NodeKind::Repo(_) => 1,
        NodeKind::Dir(children) => children.iter().map(count_repos).sum(),
    }
}

/// Render the rounded header box that appears above the tree.
pub fn header(root_path: &Path, repos: usize, elapsed: Duration, theme: Theme) -> String {
    let path_str = pretty_path(root_path);
    let ms = elapsed.as_millis();
    let title = if theme.emoji {
        format!("{} gitree", theme.header_icon())
    } else {
        "gitree".to_string()
    };

    let content = format!(
        "{title} · {path_str} · {repos} {plural} · scanned in {ms}ms",
        plural = if repos == 1 { "repo" } else { "repos" },
    );
    let content_w = UnicodeWidthStr::width(content.as_str());

    let term_w = terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(0);
    let inner = if term_w >= content_w + 4 {
        (term_w - 2).min(content_w + 4)
    } else {
        content_w + 4
    };

    let pad_right = inner.saturating_sub(content_w + 2);
    let bar = "─".repeat(inner);

    let top = format!("╭{}╮", bar);
    let mid = format!("│ {} {} │", content, " ".repeat(pad_right));
    let bot = format!("╰{}╯", bar);

    let mut out = String::new();
    out.push_str(&paint_header(&top, theme));
    out.push('\n');
    out.push_str(&paint_header_line(&mid, theme));
    out.push('\n');
    out.push_str(&paint_header(&bot, theme));
    out.push_str("\n\n");
    out
}

fn pretty_path(path: &Path) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if let Ok(home) = std::env::var("HOME") {
        if let Ok(stripped) = canonical.strip_prefix(&home) {
            let s = stripped.to_string_lossy();
            if s.is_empty() {
                return "~".to_string();
            }
            return format!("~/{}", s);
        }
    }
    canonical.display().to_string()
}

/// Build a printable, optionally colored tree.
pub fn render(root: &Node, theme: Theme) -> String {
    let mut out = String::new();

    match &root.kind {
        NodeKind::Dir(children) => {
            out.push_str(&paint_dir(&format!("{}/", root.name), theme));
            out.push('\n');
            render_children(children, "", theme, &mut out);
        }
        NodeKind::Repo(path) => {
            let info = git_info::read(path);
            let icon = icon_slot(theme.platform_icon(info.platform));
            out.push_str(&format!("{} {}\n", icon, paint_repo(&root.name, theme)));
            // Root repo: no continuation bar, just a status line indented.
            let marker = paint_dim("└─ ", theme);
            out.push_str(&format!("   {}{}\n", marker, build_status(&info, theme)));
        }
    }
    out
}

fn render_children(children: &[Node], prefix: &str, theme: Theme, out: &mut String) {
    let infos: Vec<Option<RepoInfo>> = children
        .iter()
        .map(|c| match &c.kind {
            NodeKind::Repo(p) => Some(git_info::read(p)),
            _ => None,
        })
        .collect();

    let last_idx = children.len().saturating_sub(1);
    for (i, (child, info)) in children.iter().zip(infos.iter()).enumerate() {
        let is_last = i == last_idx;
        let connector_raw = if is_last { "└─ " } else { "├─ " };
        let connector = paint_dim(connector_raw, theme);

        match (&child.kind, info) {
            (NodeKind::Repo(_), Some(info)) => {
                // Line 1: connector + icon slot (2 cells) + space + name
                let icon = icon_slot(theme.platform_icon(info.platform));
                let name = paint_repo(&child.name, theme);
                out.push_str(&format!("{prefix}{connector}{icon} {name}\n"));

                // Line 2: continuation that keeps the repo's vertical line
                // alive for siblings below, then a tree-shaped marker, then
                // the status text. Indent is 5 + 3 = 8 columns from `prefix`
                // — gives a hanging-indent feel under the repo name.
                let cont_raw = if is_last { "     " } else { "│    " };
                let cont = paint_dim(cont_raw, theme);
                let marker = paint_dim("└─ ", theme);
                out.push_str(&format!(
                    "{prefix}{cont}{marker}{}\n",
                    build_status(info, theme),
                ));
            }
            (NodeKind::Dir(grand), _) => {
                let icon = icon_slot(theme.dir_icon());
                let label = paint_dir(&format!("{}/", child.name), theme);
                out.push_str(&format!("{prefix}{connector}{icon} {label}\n"));
                let next_raw = if is_last { "    " } else { "│   " };
                let next_prefix = format!("{prefix}{}", paint_dim(next_raw, theme));
                render_children(grand, &next_prefix, theme, out);
            }
            (NodeKind::Repo(_), None) => {}
        }
    }
}

/// Pad an icon to occupy exactly 2 display cells (the slot width) so names
/// line up across rows regardless of whether the icon is a wide emoji or a
/// narrow glyph like `▲`.
fn icon_slot(icon: &str) -> String {
    let w = UnicodeWidthStr::width(icon);
    if w >= 2 {
        icon.to_string()
    } else if w == 1 {
        format!("{} ", icon)
    } else {
        // Fallback: never empty.
        format!("{}  ", icon)
    }
}

fn build_status(info: &RepoInfo, theme: Theme) -> String {
    let marker = theme.state_marker(info.state);
    let branch = paint_branch(&info.branch, info.state, theme);
    let has_hash = matches!(info.state, RepoState::OnBranch | RepoState::Detached);

    let mut out = String::new();
    if let Some(m) = marker {
        out.push_str(&paint_state_marker(m, info.state, theme));
        out.push(' ');
    }
    out.push_str(&branch);
    if has_hash {
        out.push(' ');
        out.push_str(&paint_dim("·", theme));
        out.push(' ');
        out.push_str(&paint_hash(&info.short_hash, theme));
    }
    out
}

// --- color helpers -----------------------------------------------------------

fn paint_header(s: &str, theme: Theme) -> String {
    if theme.color {
        s.bright_cyan().to_string()
    } else {
        s.to_string()
    }
}

fn paint_header_line(s: &str, theme: Theme) -> String {
    if theme.color {
        s.bright_cyan().bold().to_string()
    } else {
        s.to_string()
    }
}

fn paint_dir(s: &str, theme: Theme) -> String {
    if theme.color {
        s.bright_cyan().bold().to_string()
    } else {
        s.to_string()
    }
}

fn paint_repo(s: &str, theme: Theme) -> String {
    if theme.color {
        s.bold().to_string()
    } else {
        s.to_string()
    }
}

fn paint_branch(s: &str, state: RepoState, theme: Theme) -> String {
    if !theme.color {
        return s.to_string();
    }
    match state {
        RepoState::OnBranch => s.green().to_string(),
        RepoState::Detached => s.red().bold().to_string(),
        RepoState::Empty => s.yellow().dimmed().to_string(),
        RepoState::Unreadable => s.bright_black().to_string(),
    }
}

fn paint_state_marker(s: &str, state: RepoState, theme: Theme) -> String {
    if !theme.color {
        return s.to_string();
    }
    match state {
        RepoState::OnBranch => s.to_string(),
        RepoState::Detached => s.red().bold().to_string(),
        RepoState::Empty => s.yellow().to_string(),
        RepoState::Unreadable => s.bright_black().to_string(),
    }
}

fn paint_hash(s: &str, theme: Theme) -> String {
    if theme.color {
        s.yellow().dimmed().to_string()
    } else {
        s.to_string()
    }
}

/// Generic dim color — used for tree glyphs and the status separator `·`.
fn paint_dim(s: &str, theme: Theme) -> String {
    if theme.color {
        s.bright_black().to_string()
    } else {
        s.to_string()
    }
}
