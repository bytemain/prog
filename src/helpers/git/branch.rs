use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn resolve_gitdir(repo_path: &str) -> Option<PathBuf> {
    let dot_git = Path::new(repo_path).join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    if dot_git.is_file() {
        if let Ok(contents) = fs::read_to_string(&dot_git) {
            for line in contents.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("gitdir:") {
                    let dir = rest.trim();
                    let pb = PathBuf::from(dir);
                    if pb.is_relative() {
                        return Some(Path::new(repo_path).join(dir));
                    } else {
                        return Some(pb);
                    }
                }
            }
        }
    }
    None
}

fn read_branch_from_gitdir(gitdir: &Path) -> Option<String> {
    let head_path = gitdir.join("HEAD");
    if let Ok(contents) = fs::read_to_string(head_path) {
        let trimmed = contents.trim();
        if let Some(rest) = trimmed.strip_prefix("ref:") {
            let ref_path = rest.trim();
            // Remove common ref prefixes to get the actual branch name
            if let Some(branch_name) = ref_path.strip_prefix("refs/heads/") {
                return Some(branch_name.to_string());
            } else if let Some(branch_name) = ref_path.strip_prefix("refs/remotes/") {
                return Some(branch_name.to_string());
            } else if let Some(tag_name) = ref_path.strip_prefix("refs/tags/") {
                return Some(format!("tags/{}", tag_name));
            } else {
                // Fallback: return the full ref path for unknown formats
                return Some(ref_path.to_string());
            }
        } else {
            // Detached HEAD; return short commit id if looks like SHA, else a marker
            let short = trimmed.chars().take(7).collect::<String>();
            return Some(format!("detached-{}", short));
        }
    }
    None
}

pub fn get_branch(repo_path: &str) -> String {
    resolve_gitdir(repo_path)
        .and_then(|gitdir| read_branch_from_gitdir(&gitdir))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_gitdir_with_head(head_content: &str) -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        let gitdir = temp_dir.path().join(".git");
        fs::create_dir_all(&gitdir).unwrap();

        let head_path = gitdir.join("HEAD");
        fs::write(head_path, head_content).unwrap();

        temp_dir
    }

    #[test]
    fn test_read_branch_simple_branch() {
        let temp_dir = create_test_gitdir_with_head("ref: refs/heads/main\n");
        let gitdir = temp_dir.path().join(".git");

        let branch = read_branch_from_gitdir(&gitdir);
        assert_eq!(branch, Some("main".to_string()));
    }

    #[test]
    fn test_read_branch_with_slash() {
        let temp_dir = create_test_gitdir_with_head("ref: refs/heads/release/1.100\n");
        let gitdir = temp_dir.path().join(".git");

        let branch = read_branch_from_gitdir(&gitdir);
        assert_eq!(branch, Some("release/1.100".to_string()));
    }

    #[test]
    fn test_read_branch_feature_branch() {
        let temp_dir = create_test_gitdir_with_head("ref: refs/heads/feature/user-auth\n");
        let gitdir = temp_dir.path().join(".git");

        let branch = read_branch_from_gitdir(&gitdir);
        assert_eq!(branch, Some("feature/user-auth".to_string()));
    }

    #[test]
    fn test_read_branch_remote_branch() {
        let temp_dir = create_test_gitdir_with_head("ref: refs/remotes/origin/main\n");
        let gitdir = temp_dir.path().join(".git");

        let branch = read_branch_from_gitdir(&gitdir);
        assert_eq!(branch, Some("origin/main".to_string()));
    }

    #[test]
    fn test_read_branch_detached_head() {
        let temp_dir = create_test_gitdir_with_head("1234567890abcdef1234567890abcdef12345678\n");
        let gitdir = temp_dir.path().join(".git");

        let branch = read_branch_from_gitdir(&gitdir);
        assert_eq!(branch, Some("detached-1234567".to_string()));
    }

    #[test]
    fn test_read_branch_tag() {
        let temp_dir = create_test_gitdir_with_head("ref: refs/tags/v1.0.0\n");
        let gitdir = temp_dir.path().join(".git");

        let branch = read_branch_from_gitdir(&gitdir);
        assert_eq!(branch, Some("tags/v1.0.0".to_string()));
    }

    #[test]
    fn test_read_branch_unknown_ref_format() {
        let temp_dir = create_test_gitdir_with_head("ref: refs/custom/something\n");
        let gitdir = temp_dir.path().join(".git");

        let branch = read_branch_from_gitdir(&gitdir);
        assert_eq!(branch, Some("refs/custom/something".to_string()));
    }
}
