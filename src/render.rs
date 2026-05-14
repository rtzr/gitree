use unicode_width::UnicodeWidthStr;

use crate::git_info::{self, RepoInfo};
use crate::scan::{Node, NodeKind};

/// Build a printable tree string from a scanned node.
pub fn render(root: &Node) -> String {
    let mut out = String::new();
    out.push_str(&label(root));
    out.push('\n');

    match &root.kind {
        NodeKind::Dir(children) => render_children(children, "", &mut out),
        NodeKind::Repo(path) => {
            let info = git_info::read(path);
            out.push_str(&format!("  {} @ {}\n", info.branch, info.short_hash));
        }
    }
    out
}

fn render_children(children: &[Node], prefix: &str, out: &mut String) {
    let infos: Vec<Option<RepoInfo>> = children
        .iter()
        .map(|c| match &c.kind {
            NodeKind::Repo(p) => Some(git_info::read(p)),
            _ => None,
        })
        .collect();

    let name_w = children
        .iter()
        .map(|c| UnicodeWidthStr::width(label(c).as_str()))
        .max()
        .unwrap_or(0);
    let branch_w = infos
        .iter()
        .filter_map(|o| o.as_ref())
        .map(|i| UnicodeWidthStr::width(i.branch.as_str()))
        .max()
        .unwrap_or(0);

    let last_idx = children.len().saturating_sub(1);
    for (i, (child, info)) in children.iter().zip(infos.iter()).enumerate() {
        let is_last = i == last_idx;
        let connector = if is_last { "└─ " } else { "├─ " };
        let lbl = label(child);
        let lbl_w = UnicodeWidthStr::width(lbl.as_str());
        let name_pad = " ".repeat(name_w.saturating_sub(lbl_w));

        match (&child.kind, info) {
            (NodeKind::Repo(_), Some(info)) => {
                let bw = UnicodeWidthStr::width(info.branch.as_str());
                let branch_pad = " ".repeat(branch_w.saturating_sub(bw));
                out.push_str(&format!(
                    "{prefix}{connector}{lbl}{name_pad}   {}{branch_pad}   {}\n",
                    info.branch, info.short_hash,
                ));
            }
            (NodeKind::Dir(grand), _) => {
                out.push_str(&format!("{prefix}{connector}{lbl}\n"));
                let next_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
                render_children(grand, &next_prefix, out);
            }
            // Repo with None info: should not happen, but skip defensively.
            (NodeKind::Repo(_), None) => {}
        }
    }
}

fn label(node: &Node) -> String {
    match &node.kind {
        NodeKind::Dir(_) => format!("{}/", node.name),
        NodeKind::Repo(_) => node.name.clone(),
    }
}
