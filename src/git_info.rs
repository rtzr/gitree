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

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{RepositoryInitOptions, Signature};
    use std::path::Path;
    use tempfile::TempDir;

    /// Build a single empty commit on the current HEAD ref. Returns its oid.
    fn commit_empty(repo: &Repository, message: &str) -> git2::Oid {
        let sig = Signature::now("gitree-test", "test@example.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        let parents: Vec<git2::Commit> = repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok())
            .into_iter()
            .collect();
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
            .unwrap()
    }

    /// Initialize a repo with a deterministic initial branch name so tests
    /// don't depend on the host's `init.defaultBranch`.
    fn init_with_branch(dir: &Path, branch: &str) -> Repository {
        let mut opts = RepositoryInitOptions::new();
        opts.initial_head(branch);
        Repository::init_opts(dir, &opts).unwrap()
    }

    #[test]
    fn on_branch_parses_branch_and_short_hash() {
        let dir = TempDir::new().unwrap();
        let repo = init_with_branch(dir.path(), "main");
        let oid = commit_empty(&repo, "initial");

        let info = read(dir.path());

        assert_eq!(info.state, RepoState::OnBranch);
        assert_eq!(info.branch, "main");
        assert_eq!(info.short_hash.len(), HASH_LEN);
        // short_hash should be the first 6 chars of the commit oid.
        let full = oid.to_string();
        assert_eq!(info.short_hash, full[..HASH_LEN]);
    }

    #[test]
    fn on_branch_picks_up_non_default_branch_name() {
        let dir = TempDir::new().unwrap();
        let repo = init_with_branch(dir.path(), "main");
        commit_empty(&repo, "first");

        // Move to a feature branch.
        let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature/login", &head_commit, false).unwrap();
        repo.set_head("refs/heads/feature/login").unwrap();

        let info = read(dir.path());
        assert_eq!(info.state, RepoState::OnBranch);
        assert_eq!(info.branch, "feature/login");
    }

    #[test]
    fn empty_repo_is_reported_as_empty() {
        let dir = TempDir::new().unwrap();
        let _repo = init_with_branch(dir.path(), "main");
        // No commits at all.

        let info = read(dir.path());
        assert_eq!(info.state, RepoState::Empty);
        assert_eq!(info.branch, "(no commits)");
        // Placeholder hash, not a real one.
        assert!(info.short_hash.chars().all(|c| c == '-'));
    }

    #[test]
    fn detached_head_is_reported_as_detached() {
        let dir = TempDir::new().unwrap();
        let repo = init_with_branch(dir.path(), "main");
        let oid = commit_empty(&repo, "first");
        // Detach HEAD onto the commit directly.
        repo.set_head_detached(oid).unwrap();

        let info = read(dir.path());
        assert_eq!(info.state, RepoState::Detached);
        assert_eq!(info.branch, "(detached)");
        // The hash should still be the commit we detached onto.
        assert_eq!(info.short_hash, oid.to_string()[..HASH_LEN]);
    }

    #[test]
    fn non_repo_directory_is_unreadable() {
        let dir = TempDir::new().unwrap();
        // A plain directory with no .git — Repository::open should fail.

        let info = read(dir.path());
        assert_eq!(info.state, RepoState::Unreadable);
        assert_eq!(info.branch, "???");
        assert!(info.short_hash.chars().all(|c| c == '?'));
    }
}
