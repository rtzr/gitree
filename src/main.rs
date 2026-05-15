mod git_info;
mod platform;
mod progress;
mod render;
mod scan;

use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::Parser;

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
    let theme = render::Theme::resolve(args.no_color, args.no_emoji);

    let spinner = progress::Spinner::start(
        format!("Scanning {}…", args.path.display()),
        theme.color,
        theme.emoji,
    );

    let root_for_report = args.path.clone();
    let report = |p: &Path| {
        spinner.set_status(format!("Scanning {}…", display_progress(&root_for_report, p)));
    };

    let started = Instant::now();
    let scanned = scan::scan(&args.path, args.depth, args.all, &report);
    // Git info is read lazily during render; keep the spinner running so the
    // user sees activity until the final output is ready.
    spinner.set_status("Reading git info…".to_string());
    let rendered = scanned.as_ref().map(|root| render::render(root, theme));
    let elapsed = started.elapsed();

    spinner.stop();

    match (scanned, rendered) {
        (Some(root), Some(tree)) => {
            let repos = render::count_repos(&root);
            print!("{}", render::header(&args.path, repos, elapsed, theme));
            print!("{}", tree);
        }
        _ => {
            println!("(no git repositories found under {})", args.path.display());
        }
    }
}

/// Format a path relative to the scan root for the spinner status line.
/// If the path is the root itself, show the root's basename; otherwise show
/// the relative tail so the user sees concrete progress.
fn display_progress(root: &Path, current: &Path) -> String {
    if current == root {
        return root
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.display().to_string());
    }
    if let Ok(rel) = current.strip_prefix(root) {
        return rel.display().to_string();
    }
    current.display().to_string()
}
