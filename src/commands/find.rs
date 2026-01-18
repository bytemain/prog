use crate::{
    context::{
        Context,
        database::{MatchKind, MatchedRepo},
    },
    helpers::{git, path, platform},
};
use inquire::Select;
use std::collections::HashSet;

use super::printer::error::handle_inquire_error;

use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Debug)]
pub struct FoundItem {
    pub file_path: String,
    pub branch: String,
    pub match_hint: Option<String>,
}

impl Display for FoundItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let display_path = path::contract_tilde(&self.file_path);
        let base = if self.branch.trim().is_empty() {
            display_path
        } else {
            format!("{} @{}", display_path, self.branch)
        };

        if let Some(hint) = self.match_hint.as_ref().filter(|hint| !hint.trim().is_empty()) {
            write!(f, "{} ({})", base, hint)
        } else {
            write!(f, "{}", base)
        }
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
) -> Option<String> {
    match match_kind {
        MatchKind::PathContains => None,
        MatchKind::RepoExact | MatchKind::RepoContains => Some(format!("repo: {}", repo.repo)),
        MatchKind::FullNameExact
        | MatchKind::OwnerExact
        | MatchKind::OwnerContains
        | MatchKind::RemoteContains => {
            Some(format!("remote: {}/{}/{}", repo.host, repo.owner, repo.repo))
        }
    }
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
                    match_hint: match_hint(repo, matched.match_kind),
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
            full_path: "/base/pyenv-versions".to_string(),
        };

        let hint = match_hint(&repo, MatchKind::OwnerExact);

        assert_eq!(hint, Some("remote: github.com/version-fox/versa-vault".to_string()));
    }

    #[test]
    fn test_found_item_display_with_hint() {
        let item = FoundItem {
            file_path: "/tmp/repo".to_string(),
            branch: "main".to_string(),
            match_hint: Some("repo: prog".to_string()),
        };

        assert_eq!(format!("{}", item), "/tmp/repo @main (repo: prog)");
    }
}
