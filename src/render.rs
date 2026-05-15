use std::io::IsTerminal;
use std::path::Path;
use std::time::Duration;

use owo_colors::OwoColorize;
use unicode_width::UnicodeWidthStr;

use crate::git_info::{self, RepoInfo, RepoState};
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

    fn repo_icon(self) -> &'static str {
        if self.emoji { "📦" } else { "[G]" }
    }

    fn header_icon(self) -> &'static str {
        if self.emoji { "🌳" } else { "gitree" }
    }

    fn state_dot(self, state: RepoState) -> &'static str {
        if !self.emoji {
            return match state {
                RepoState::OnBranch => "*",
                RepoState::Detached => "!",
                RepoState::Empty => "o",
                RepoState::Unreadable => "?",
            };
        }
        match state {
            RepoState::OnBranch => "●",
            RepoState::Detached => "⚠",
            RepoState::Empty => "○",
            RepoState::Unreadable => "✕",
        }
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

    // Pad to terminal width if available, otherwise just content width + breathing room.
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

fn paint_header(s: &str, theme: Theme) -> String {
    if theme.color {
        s.bright_cyan().to_string()
    } else {
        s.to_string()
    }
}

fn paint_header_line(s: &str, theme: Theme) -> String {
    // Border characters stay cyan but the content is bold for readability.
    // owo-colors doesn't easily mix per-substring, so we approximate: just bright_cyan whole line.
    if theme.color {
        s.bright_cyan().bold().to_string()
    } else {
        s.to_string()
    }
}

/// Build a printable, optionally colored tree.
pub fn render(root: &Node, theme: Theme) -> String {
    let mut out = String::new();
    let lbl = root_label(root, theme);
    out.push_str(&lbl);
    out.push('\n');

    match &root.kind {
        NodeKind::Dir(children) => render_children(children, "", theme, &mut out),
        NodeKind::Repo(path) => {
            let info = git_info::read(path);
            let dot = theme.state_dot(info.state);
            out.push_str(&format!(
                "   {} {}  {}\n",
                paint_dot(dot, info.state, theme),
                paint_branch(&info.branch, info.state, theme),
                paint_hash(&info.short_hash, theme),
            ));
        }
    }
    out
}

fn render_children(children: &[Node], prefix: &str, theme: Theme, out: &mut String) {
    // Read git info for repo children up front so width math and printing share one source.
    let infos: Vec<Option<RepoInfo>> = children
        .iter()
        .map(|c| match &c.kind {
            NodeKind::Repo(p) => Some(git_info::read(p)),
            _ => None,
        })
        .collect();

    // Align branch labels across sibling repos so commit hashes line up.
    let branch_w = infos
        .iter()
        .filter_map(|o| o.as_ref())
        .map(|i| UnicodeWidthStr::width(i.branch.as_str()))
        .max()
        .unwrap_or(0);

    let last_idx = children.len().saturating_sub(1);
    for (i, (child, info)) in children.iter().zip(infos.iter()).enumerate() {
        let is_last = i == last_idx;
        let connector_raw = if is_last { "└─ " } else { "├─ " };
        let connector = paint_branchline(connector_raw, theme);
        let label = colored_label(child, theme);

        match (&child.kind, info) {
            (NodeKind::Repo(_), Some(info)) => {
                // Line 1: tree connector + icon + name.
                out.push_str(&format!("{prefix}{connector}{label}\n"));

                // Line 2: status indented under the repo, aligned with the
                // label's icon position. Use a vertical bar continuation for
                // non-last siblings so the eye can follow the group.
                let status_raw = if is_last { "   " } else { "│  " };
                let status_marker = paint_branchline(status_raw, theme);
                let bw = UnicodeWidthStr::width(info.branch.as_str());
                let branch_pad = " ".repeat(branch_w.saturating_sub(bw));
                let dot = theme.state_dot(info.state);
                out.push_str(&format!(
                    "{prefix}{status_marker}{} {}{branch_pad}  {}\n",
                    paint_dot(dot, info.state, theme),
                    paint_branch(&info.branch, info.state, theme),
                    paint_hash(&info.short_hash, theme),
                ));
            }
            (NodeKind::Dir(grand), _) => {
                out.push_str(&format!("{prefix}{connector}{label}\n"));
                let next_raw = if is_last { "    " } else { "│   " };
                let next_prefix = format!("{prefix}{}", paint_branchline(next_raw, theme));
                render_children(grand, &next_prefix, theme, out);
            }
            (NodeKind::Repo(_), None) => {}
        }
    }
}

// --- label helpers -----------------------------------------------------------

fn root_label(node: &Node, theme: Theme) -> String {
    match &node.kind {
        NodeKind::Dir(_) => paint_dir(&format!("{}/", node.name), theme),
        NodeKind::Repo(_) => format!(
            "{} {}",
            theme.repo_icon(),
            paint_repo(&node.name, theme),
        ),
    }
}

fn colored_label(node: &Node, theme: Theme) -> String {
    match &node.kind {
        NodeKind::Dir(_) => format!(
            "{} {}",
            theme.dir_icon(),
            paint_dir(&format!("{}/", node.name), theme),
        ),
        NodeKind::Repo(_) => format!(
            "{} {}",
            theme.repo_icon(),
            paint_repo(&node.name, theme),
        ),
    }
}

// --- color helpers -----------------------------------------------------------

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
        RepoState::Detached => s.red().to_string(),
        RepoState::Empty => s.yellow().dimmed().to_string(),
        RepoState::Unreadable => s.bright_black().to_string(),
    }
}

fn paint_dot(s: &str, state: RepoState, theme: Theme) -> String {
    if !theme.color {
        return s.to_string();
    }
    match state {
        RepoState::OnBranch => s.green().to_string(),
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

fn paint_branchline(s: &str, theme: Theme) -> String {
    if theme.color {
        s.bright_black().to_string()
    } else {
        s.to_string()
    }
}
