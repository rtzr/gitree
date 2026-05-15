use std::path::Path;

use git2::Repository;

use crate::platform::{self, Platform};

#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub branch: String,
    pub short_hash: String,
    pub state: RepoState,
    pub platform: Platform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoState {
    /// HEAD points at a named branch.
    OnBranch,
    /// HEAD is detached at a commit.
    Detached,
    /// Repo exists but has no commits yet.
    Empty,
    /// Repo could not be opened or read.
    Unreadable,
}

const HASH_LEN: usize = 6;

pub fn read(path: &Path) -> RepoInfo {
    let platform = platform::detect(path);

    let repo = match Repository::open(path) {
        Ok(r) => r,
        Err(_) => {
            return RepoInfo {
                branch: "???".into(),
                short_hash: "??????".into(),
                state: RepoState::Unreadable,
                platform,
            };
        }
    };

    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => {
            // Most commonly: a freshly initialized repo with no commits yet.
            return RepoInfo {
                branch: "(no commits)".into(),
                short_hash: "------".into(),
                state: RepoState::Empty,
                platform,
            };
        }
    };

    let oid = head
        .target()
        .or_else(|| head.peel_to_commit().ok().map(|c| c.id()));
    let short_hash = oid
        .map(|id| {
            let s = id.to_string();
            s.chars().take(HASH_LEN).collect::<String>()
        })
        .unwrap_or_else(|| "??????".into());

    if head.is_branch() {
        let branch = head.shorthand().unwrap_or("(unknown)").to_string();
        RepoInfo {
            branch,
            short_hash,
            state: RepoState::OnBranch,
            platform,
        }
    } else {
        RepoInfo {
            branch: "(detached)".into(),
            short_hash,
            state: RepoState::Detached,
            platform,
        }
    }
}
