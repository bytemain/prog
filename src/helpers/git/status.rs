use std::process::Command;

use serde::Serialize;

/// Status of a single git repository, derived from `git status --porcelain=v2 --branch`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RepoStatus {
    /// Current branch name. Empty when in detached HEAD.
    pub branch: String,
    /// True when HEAD is detached.
    pub detached: bool,
    /// Configured upstream tracking branch (e.g. "origin/main"), if any.
    pub upstream: Option<String>,
    /// Number of commits the local branch is ahead of upstream.
    pub ahead: u32,
    /// Number of commits the local branch is behind upstream.
    pub behind: u32,
    /// Number of tracked files with staged or unstaged modifications.
    pub modified: u32,
    /// Number of untracked files.
    pub untracked: u32,
    /// Number of files with merge conflicts.
    pub conflicted: u32,
}

impl RepoStatus {
    /// True when there are uncommitted changes (staged, unstaged, untracked, or conflicted).
    pub fn is_dirty(&self) -> bool {
        self.modified > 0 || self.untracked > 0 || self.conflicted > 0
    }

    /// True when the local branch has commits not present on the upstream.
    pub fn is_unpushed(&self) -> bool {
        self.ahead > 0
    }

    /// True when the branch has no upstream configured (and is not detached).
    pub fn is_no_upstream(&self) -> bool {
        !self.detached && self.upstream.is_none()
    }
}

/// Parse the output of `git status --porcelain=v2 --branch`.
///
/// Format reference: <https://git-scm.com/docs/git-status#_porcelain_format_version_2>
pub fn parse_porcelain_v2(output: &str) -> RepoStatus {
    let mut status = RepoStatus::default();

    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            let head = rest.trim();
            if head == "(detached)" {
                status.detached = true;
            } else {
                status.branch = head.to_string();
            }
        } else if let Some(rest) = line.strip_prefix("# branch.upstream ") {
            let up = rest.trim();
            if !up.is_empty() {
                status.upstream = Some(up.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            // Format: "+<ahead> -<behind>"
            let mut parts = rest.split_whitespace();
            if let Some(a) = parts.next() {
                if let Ok(n) = a.trim_start_matches('+').parse::<i64>() {
                    status.ahead = n.unsigned_abs() as u32;
                }
            }
            if let Some(b) = parts.next() {
                if let Ok(n) = b.trim_start_matches('-').parse::<i64>() {
                    status.behind = n.unsigned_abs() as u32;
                }
            }
        } else if line.starts_with("# ") {
            // Other header line; ignore.
        } else if line.starts_with("? ") {
            status.untracked += 1;
        } else if line.starts_with("u ") {
            status.conflicted += 1;
        } else if line.starts_with("1 ") || line.starts_with("2 ") {
            // Ordinary or renamed/copied changed entry.
            status.modified += 1;
        }
    }

    status
}

/// Run `git status --porcelain=v2 --branch` in `repo_path` and return parsed status.
pub fn get_repo_status(repo_path: &str) -> Option<RepoStatus> {
    let output = Command::new("git")
        .args(["status", "--porcelain=v2", "--branch"])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(parse_porcelain_v2(&stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_clean_synced() {
        let out = "\
# branch.oid abcdef1234567890
# branch.head main
# branch.upstream origin/main
# branch.ab +0 -0
";
        let s = parse_porcelain_v2(out);
        assert_eq!(s.branch, "main");
        assert_eq!(s.upstream.as_deref(), Some("origin/main"));
        assert_eq!(s.ahead, 0);
        assert_eq!(s.behind, 0);
        assert_eq!(s.modified, 0);
        assert_eq!(s.untracked, 0);
        assert!(!s.detached);
        assert!(!s.is_dirty());
        assert!(!s.is_unpushed());
        assert!(!s.is_no_upstream());
    }

    #[test]
    fn parse_ahead_and_dirty() {
        let out = "\
# branch.oid abcdef1234567890
# branch.head feature/x
# branch.upstream origin/feature/x
# branch.ab +3 -1
1 .M N... 100644 100644 100644 aaa bbb file1.rs
1 M. N... 100644 100644 100644 aaa bbb file2.rs
2 R. N... 100644 100644 100644 aaa bbb R100 new.rs\told.rs
? untracked.txt
? another.txt
";
        let s = parse_porcelain_v2(out);
        assert_eq!(s.branch, "feature/x");
        assert_eq!(s.upstream.as_deref(), Some("origin/feature/x"));
        assert_eq!(s.ahead, 3);
        assert_eq!(s.behind, 1);
        assert_eq!(s.modified, 3);
        assert_eq!(s.untracked, 2);
        assert!(s.is_dirty());
        assert!(s.is_unpushed());
        assert!(!s.is_no_upstream());
    }

    #[test]
    fn parse_no_upstream() {
        let out = "\
# branch.oid abcdef1234567890
# branch.head local-only
";
        let s = parse_porcelain_v2(out);
        assert_eq!(s.branch, "local-only");
        assert!(s.upstream.is_none());
        assert!(s.is_no_upstream());
        assert!(!s.detached);
    }

    #[test]
    fn parse_detached_head() {
        let out = "\
# branch.oid abcdef1234567890
# branch.head (detached)
";
        let s = parse_porcelain_v2(out);
        assert!(s.detached);
        assert!(s.branch.is_empty());
        assert!(!s.is_no_upstream());
    }

    #[test]
    fn parse_conflicted() {
        let out = "\
# branch.oid abcdef1234567890
# branch.head main
# branch.upstream origin/main
# branch.ab +0 -0
u UU N... 100644 100644 100644 100644 aaa bbb ccc conflict.rs
";
        let s = parse_porcelain_v2(out);
        assert_eq!(s.conflicted, 1);
        assert!(s.is_dirty());
    }
}
