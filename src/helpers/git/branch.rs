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
            return Some(ref_path.split('/').last().unwrap_or(ref_path).to_string());
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
