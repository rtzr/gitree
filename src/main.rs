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

    /// Number of threads for the parallel directory scan. `0` means auto
    /// (= number of CPU cores). Default is auto, except on Linux when a
    /// rotational disk (HDD) is detected — then defaults to 1 to avoid
    /// head thrashing.
    #[arg(short = 'j', long = "jobs")]
    jobs: Option<usize>,
}

fn main() {
    let args = Args::parse();
    let theme = render::Theme::resolve(args.no_color, args.no_emoji);

    // Decide parallelism before starting any FS work. `threads = 1` ⇒ serial
    // path (no rayon overhead). `> 1` ⇒ resize the global pool. `0` ⇒ leave
    // rayon's default pool size (= num_cpus) untouched.
    let threads = pick_thread_count(args.jobs, &args.path);
    let parallel = threads != 1;
    if threads > 1 {
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global();
    }

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
    let scanned = scan::scan(&args.path, args.depth, args.all, &report, parallel);
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

/// Decide how many threads to use for the scan.
///
/// - User-specified `-j N` wins outright (`-j 0` = explicit auto).
/// - Otherwise: auto (return 0 ⇒ rayon's default = num_cpus), **except** on
///   Linux when the target lives on a rotational disk — then return 1 to
///   avoid HDD seek thrashing.
fn pick_thread_count(user_jobs: Option<usize>, scan_root: &Path) -> usize {
    if let Some(n) = user_jobs {
        return n;
    }
    #[cfg(target_os = "linux")]
    {
        if is_rotational_disk(scan_root).unwrap_or(false) {
            return 1;
        }
    }
    let _ = scan_root; // suppress unused on non-Linux
    0
}

/// Linux-only: does `path` live on a rotational (HDD) block device?
///
/// Maps path → st_dev → `/sys/dev/block/M:m` → walks up to find
/// `queue/rotational` (1 = HDD, 0 = SSD/NVMe). Returns None if anything in
/// that chain isn't readable — we then treat the disk as non-rotational
/// (the optimistic default).
#[cfg(target_os = "linux")]
fn is_rotational_disk(path: &Path) -> Option<bool> {
    use std::os::unix::fs::MetadataExt;

    let meta = std::fs::metadata(path).ok()?;
    let dev = meta.dev();
    // glibc's gnu_dev_major / gnu_dev_minor bit layout.
    let major = ((dev >> 8) & 0xfff) | ((dev >> 32) & !0xfff);
    let minor = (dev & 0xff) | ((dev >> 12) & !0xff);

    let dev_link = format!("/sys/dev/block/{}:{}", major, minor);
    let mut cur = std::fs::canonicalize(&dev_link).ok()?;
    loop {
        let rot = cur.join("queue/rotational");
        if let Ok(s) = std::fs::read_to_string(&rot) {
            return Some(s.trim() == "1");
        }
        if !cur.pop() {
            return None;
        }
    }
}
