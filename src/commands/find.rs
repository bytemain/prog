use crate::{
    context::{
        Context,
        database::{MatchKind, MatchedRepo},
    },
    helpers::{git, path, platform},
};
use inquire::Select;
use std::collections::HashSet;
use std::path::Path;

use super::printer::error::handle_inquire_error;

use std::fmt::{Display, Formatter, Result as FmtResult};

const BRANCH_PADDING: usize = 2;

#[derive(Clone, Debug)]
pub struct FoundItem {
    pub file_path: String,
    pub branch: String,
    pub match_hint: Option<String>,
    pub display_label: Option<String>,
}

impl Display for FoundItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(display_label) = self.display_label.as_ref() {
            return write!(f, "{}", display_label);
        }

        let display_path = path::contract_tilde(&self.file_path);
        let base = build_left_label(&display_path, self.match_hint.as_deref());
        format_display_line(&base, &self.branch, base.chars().count()).fmt(f)
    }
}

fn handle_result(item: &FoundItem) {
    println!("Found: {}", item);
    platform::clipboard::copy_path(&item.file_path);
}

fn print_found_item_path(item: &FoundItem) {
    println!("{}", item.file_path);
}

/// Extracts a search term from the input.
/// If the input looks like a git URL, extracts the fullname (owner/repo) from it.
/// Otherwise, returns the original input as-is.
fn extract_search_term(input: &str) -> String {
    if let Some(parsed) = git::parse_git_url(input) {
        if git::remote_url_is_valid(&parsed) {
            return parsed.fullname;
        }
    }
    input.to_string()
}

fn match_hint(
    repo: &crate::context::database::models::Repo,
    match_kind: MatchKind,
    file_path: &str,
) -> Option<String> {
    let repo_name = repo.repo.to_lowercase();
    let repo_folder_matches = Path::new(file_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase() == repo_name)
        .unwrap_or(false);

    match match_kind {
        MatchKind::PathContains => None,
        MatchKind::RepoExact | MatchKind::RepoContains => {
            if repo_folder_matches {
                None
            } else {
                Some(format!("repo: {}", repo.repo))
            }
        }
        MatchKind::FullNameExact => {
            Some(format!("remote: {}/{}/{}", repo.host, repo.owner, repo.repo))
        }
        MatchKind::OwnerExact | MatchKind::OwnerContains | MatchKind::RemoteContains => {
            Some(format!("remote: {}/{}/{}", repo.host, repo.owner, repo.repo))
        }
    }
}

/// Builds the left-side label shown in selection lists, including any match hint.
fn build_left_label(display_path: &str, match_hint: Option<&str>) -> String {
    if let Some(hint) = match_hint.filter(|hint| !hint.trim().is_empty()) {
        format!("{} ({})", display_path, hint)
    } else {
        display_path.to_string()
    }
}

/// Formats a full display line with optional branch information aligned to `max_width`.
fn format_display_line(base: &str, branch: &str, max_width: usize) -> String {
    let base_len = base.chars().count();
    format_display_line_with_len(base, branch, max_width, base_len)
}

fn format_display_line_with_len(
    base: &str,
    branch: &str,
    max_width: usize,
    base_len: usize,
) -> String {
    if branch.trim().is_empty() {
        return base.to_string();
    }

    let padding = max_width.saturating_sub(base_len) + BRANCH_PADDING;
    format!("{}{}[{}]", base, " ".repeat(padding), branch)
}

pub fn find_keyword(c: &Context, keyword: &str) -> Option<Vec<FoundItem>> {
    c.auto_sync_silent();

    let search_term = extract_search_term(keyword);
    let result: Vec<MatchedRepo> = c.database_mut().find(&search_term);
    if result.is_empty() {
        return None;
    }

    // Use Vec with HashSet for deduplication while preserving insertion order
    let mut options: Vec<FoundItem> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let mut should_sync = false;
    for matched in result {
        let repo = &matched.repo;
        let path_str: String = repo.full_path.clone();
        if path::exists(&path_str) {
            // Repo path entry with branch
            if seen.insert(path_str.clone()) {
                options.push(FoundItem {
                    file_path: path_str.clone(),
                    branch: git::get_branch(&path_str),
                    match_hint: match_hint(repo, matched.match_kind, &path_str),
                    display_label: None,
                });
            }

            // Host directory entry (no branch)
            if repo.host == keyword {
                let host_path = repo.host_fs_path();
                if seen.insert(host_path.clone()) {
                    options.push(FoundItem {
                        file_path: host_path,
                        branch: String::new(),
                        match_hint: None,
                        display_label: None,
                    });
                }
            }

            // Owner directory entry (no branch)
            if repo.owner == keyword {
                let owner_path = repo.owner_fs_path();
                if seen.insert(owner_path.clone()) {
                    options.push(FoundItem {
                        file_path: owner_path,
                        branch: String::new(),
                        match_hint: None,
                        display_label: None,
                    });
                }
            }
        } else {
            should_sync = true;
        }
    }

    if should_sync {
        c.sync_silent();
    }

    let mut max_width = 0;
    let display_paths: Vec<(String, usize)> = options
        .iter()
        .map(|item| {
            let label = build_left_label(
                &path::contract_tilde(&item.file_path),
                item.match_hint.as_deref(),
            );
            let label_len = label.chars().count();
            max_width = max_width.max(label_len);
            (label, label_len)
        })
        .collect();
    for (item, (display_path, label_len)) in options.iter_mut().zip(display_paths) {
        item.display_label =
            Some(format_display_line_with_len(&display_path, &item.branch, max_width, label_len));
    }

    Some(options)
}

pub fn run(c: &Context, keyword: &str, _query: bool) {
    if _query {
        query(&c, &keyword);
    } else {
        find(&c, &keyword);
    }
}

pub fn query(c: &Context, keyword: &str) {
    let result = find_keyword(c, keyword).unwrap_or_default();

    if result.is_empty() {
        return;
    }

    if result.len() == 1 {
        print_found_item_path(&result[0]);
        return;
    }

    let ans = Select::new("Which project are you looking for?", result.clone()).prompt();

    match ans {
        Ok(choice) => {
            print_found_item_path(&choice);
        }
        Err(e) => handle_inquire_error(e),
    }
}

pub fn find(c: &Context, keyword: &str) -> bool {
    let result = find_keyword(c, keyword).unwrap_or_default();

    if result.is_empty() {
        return false;
    }

    if result.len() == 1 {
        handle_result(&result[0]);
        return true;
    }

    let ans = Select::new("Which project are you looking for?", result.clone()).prompt();

    match ans {
        Ok(choice) => {
            handle_result(&choice);
            return true;
        }
        Err(e) => handle_inquire_error(e),
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_search_term_scp_like_url() {
        let result = extract_search_term("git@github.com:bytemain/prog.git");
        assert_eq!(result, "bytemain/prog");
    }

    #[test]
    fn test_extract_search_term_https_url() {
        let result = extract_search_term("https://github.com/bytemain/prog.git");
        assert_eq!(result, "bytemain/prog");
    }

    #[test]
    fn test_extract_search_term_https_url_without_git_suffix() {
        let result = extract_search_term("https://github.com/bytemain/prog");
        assert_eq!(result, "bytemain/prog");
    }

    #[test]
    fn test_extract_search_term_ssh_url() {
        let result = extract_search_term("ssh://git@github.com/owner/repo");
        assert_eq!(result, "owner/repo");
    }

    #[test]
    fn test_extract_search_term_plain_keyword() {
        let result = extract_search_term("prog");
        assert_eq!(result, "prog");
    }

    #[test]
    fn test_extract_search_term_plain_keyword_with_slash() {
        // This could be interpreted as owner/repo, but no host, so it's not a valid git URL
        let result = extract_search_term("bytemain/prog");
        // Note: parse_git_url parses this as host="bytemain", owner="prog", but repo is empty
        // so remote_url_is_valid returns false and we fall back to the input
        assert_eq!(result, "bytemain/prog");
    }

    #[test]
    fn test_extract_search_term_empty_string() {
        let result = extract_search_term("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_extract_search_term_partial_url_invalid() {
        // Invalid URL without owner/repo should return original
        let result = extract_search_term("git@github.com:owner");
        assert_eq!(result, "git@github.com:owner");
    }

    #[test]
    fn test_match_hint_owner_match_includes_remote() {
        let now = chrono::Utc::now().naive_utc();
        let repo = crate::context::database::models::Repo {
            created_at: now,
            updated_at: now,
            host: "github.com".to_string(),
            repo: "versa-vault".to_string(),
            owner: "version-fox".to_string(),
            remote_url: "git@github.com:version-fox/versa-vault.git".to_string(),
            base_dir: "/base".to_string(),
            // Non-standard path to mirror repositories cloned outside owner/repo structure.
            full_path: "/base/pyenv-versions".to_string(),
        };

        let hint = match_hint(&repo, MatchKind::OwnerExact, &repo.full_path);

        assert_eq!(hint, Some("remote: github.com/version-fox/versa-vault".to_string()));
    }

    #[test]
    fn test_found_item_display_with_hint() {
        let item = FoundItem {
            file_path: "/tmp/repo".to_string(),
            branch: "main".to_string(),
            match_hint: Some("repo: prog".to_string()),
            display_label: None,
        };

        assert_eq!(format!("{}", item), "/tmp/repo (repo: prog)  [main]");
    }

    #[test]
    fn test_match_hint_repo_name_matches_folder_skips_hint() {
        let now = chrono::Utc::now().naive_utc();
        let repo = crate::context::database::models::Repo {
            created_at: now,
            updated_at: now,
            host: "github.com".to_string(),
            repo: "prog".to_string(),
            owner: "bytemain".to_string(),
            remote_url: "https://github.com/bytemain/prog.git".to_string(),
            base_dir: "/base".to_string(),
            full_path: "/base/github.com/bytemain/prog".to_string(),
        };

        let hint = match_hint(&repo, MatchKind::RepoExact, &repo.full_path);

        assert_eq!(hint, None);
    }
}
