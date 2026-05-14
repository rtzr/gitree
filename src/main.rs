mod git_info;
mod scan;

use std::path::PathBuf;

use clap::Parser;

use crate::scan::{Node, NodeKind};

/// Tree view of git repositories under a directory.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Root path to scan. Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Limit the tree to N levels deep.
    #[arg(short = 'L', long)]
    depth: Option<usize>,

    /// Disable ANSI colors (also auto-disabled when stdout isn't a TTY).
    #[arg(long)]
    no_color: bool,

    /// Don't skip default-ignored directories (node_modules, target, dot-prefixed, ...).
    #[arg(short = 'a', long)]
    all: bool,

    /// Use ASCII markers instead of emoji.
    #[arg(long)]
    no_emoji: bool,
}

fn main() {
    let args = Args::parse();
    match scan::scan(&args.path, args.depth, args.all) {
        Some(root) => print_debug(&root, 0),
        None => println!("(no git repositories found under {})", args.path.display()),
    }
}

fn print_debug(node: &Node, depth: usize) {
    let pad = "  ".repeat(depth);
    match &node.kind {
        NodeKind::Repo(path) => {
            let info = git_info::read(path);
            println!(
                "{pad}- {} [{} @ {}]",
                node.name, info.branch, info.short_hash
            );
        }
        NodeKind::Dir(children) => {
            println!("{pad}+ {}", node.name);
            for c in children {
                print_debug(c, depth + 1);
            }
        }
    }
}
