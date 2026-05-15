use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

const DEFAULT_SKIP: &[&str] = &[
    "node_modules",
    "target",
    "dist",
    "build",
    "vendor",
    "__pycache__",
    ".next",
    ".nuxt",
    ".turbo",
    ".cache",
];

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub kind: NodeKind,
}

#[derive(Debug)]
pub enum NodeKind {
    Dir(Vec<Node>),
    Repo(PathBuf),
}

pub fn scan(
    root: &Path,
    max_depth: Option<usize>,
    include_all: bool,
    reporter: &(dyn Fn(&Path) + Sync),
    parallel: bool,
) -> Option<Node> {
    let name = root
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.display().to_string());
    walk(root, &name, 0, max_depth, include_all, reporter, parallel)
}

fn walk(
    path: &Path,
    name: &str,
    depth: usize,
    max_depth: Option<usize>,
    include_all: bool,
    reporter: &(dyn Fn(&Path) + Sync),
    parallel: bool,
) -> Option<Node> {
    reporter(path);

    if is_repo(path) {
        return Some(Node {
            name: name.to_string(),
            kind: NodeKind::Repo(path.to_path_buf()),
        });
    }

    if let Some(max) = max_depth {
        if depth >= max {
            return None;
        }
    }

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return None,
    };

    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    // par_iter().filter_map().collect() preserves input order, so sibling
    // sort order is identical to the serial path.
    let map_entry = |entry: &fs::DirEntry| -> Option<Node> {
        let ft = entry.file_type().ok()?;
        if ft.is_symlink() || !ft.is_dir() {
            return None;
        }
        let entry_name = entry.file_name().to_string_lossy().into_owned();
        if should_skip(&entry_name, include_all) {
            return None;
        }
        walk(
            &entry.path(),
            &entry_name,
            depth + 1,
            max_depth,
            include_all,
            reporter,
            parallel,
        )
    };

    let children: Vec<Node> = if parallel {
        entries.par_iter().filter_map(map_entry).collect()
    } else {
        entries.iter().filter_map(map_entry).collect()
    };

    if children.is_empty() {
        return None;
    }
    Some(Node {
        name: name.to_string(),
        kind: NodeKind::Dir(children),
    })
}

fn is_repo(path: &Path) -> bool {
    // `.git` is a directory in a normal repo, and a file inside submodules / linked worktrees.
    path.join(".git").exists()
}

fn should_skip(name: &str, include_all: bool) -> bool {
    if include_all {
        return false;
    }
    if name.starts_with('.') {
        return true;
    }
    DEFAULT_SKIP.iter().any(|s| *s == name)
}
