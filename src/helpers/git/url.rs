use std::process::Command;

#[derive(Debug, Clone)]
pub struct ParsedGitUrl {
    pub host: Option<String>,
    pub owner: Option<String>,
    pub name: String,
    pub fullname: String,
}

fn strip_git_suffix(name: &str) -> String {
    name.strip_suffix(".git").unwrap_or(name).to_string()
}

pub fn parse_git_url(input: &str) -> Option<ParsedGitUrl> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }

    // Helper to build ParsedGitUrl
    let build = |host: Option<String>, owner: Option<String>, name: String| -> ParsedGitUrl {
        let fullname = match (&owner, &name) {
            (Some(o), n) => format!("{}/{}", o, n),
            (None, n) => n.to_string(),
        };
        ParsedGitUrl { host, owner, name, fullname }
    };

    // Handle scheme-based URLs: http(s)://, ssh://, git://
    if let Some(idx) = s.find("://") {
        let mut rest = &s[idx + 3..];
        // Optional user@ prefix
        if let Some(user_at_idx) = rest.find('@') {
            // Ensure '@' appears before the first '/' (user@host/path)
            if rest[..user_at_idx].contains('/') {
                // '@' is within path; ignore as user part not present
            } else {
                rest = &rest[user_at_idx + 1..];
            }
        }
        // Split host and path
        let mut parts = rest.split('/');
        let host = parts.next().map(|h| h.to_string());
        let owner = parts.next().map(|o| o.to_string());
        let name = parts.next().map(|n| strip_git_suffix(n))?;
        return Some(build(host, owner, name));
    }

    // Handle scp-like syntax: user@host:owner/repo(.git)
    if let (Some(at_idx), Some(colon_idx)) = (s.rfind('@'), s.rfind(':')) {
        if at_idx < colon_idx {
            let host = s.get(at_idx + 1..colon_idx).map(|h| h.to_string());
            let path = &s[colon_idx + 1..];
            let mut parts = path.split('/');
            let owner = parts.next().map(|o| o.to_string());
            let name = parts.next().map(|n| strip_git_suffix(n))?;
            return Some(build(host, owner, name));
        }
    }

    // Fallback: try to parse https-like without scheme (host/owner/name)
    let mut parts = s.split('/');
    let host = parts.next().and_then(|h| if h.contains(':') || h.contains('@') { None } else { Some(h.to_string()) });
    let owner = parts.next().map(|o| o.to_string());
    if let Some(n) = parts.next() {
        return Some(build(host, owner, strip_git_suffix(n)));
    }

    None
}

pub fn get_remote_url(repo: &str) -> String {
    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(repo)
        .output()
        .expect("Failed to get remote url");

    let url = String::from_utf8(output.stdout).unwrap();
    url.trim().to_string()
}

pub fn remote_url_is_valid(parsed: &ParsedGitUrl) -> bool {
    parsed.host.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false)
        && parsed.owner.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_https_with_git_suffix() {
        let p = parse_git_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(p.host.as_deref(), Some("github.com"));
        assert_eq!(p.owner.as_deref(), Some("owner"));
        assert_eq!(p.name, "repo");
        assert_eq!(p.fullname, "owner/repo");
        assert!(remote_url_is_valid(&p));
    }

    #[test]
    fn parse_https_without_git_suffix() {
        let p = parse_git_url("https://github.com/owner/repo").unwrap();
        assert_eq!(p.host.as_deref(), Some("github.com"));
        assert_eq!(p.owner.as_deref(), Some("owner"));
        assert_eq!(p.name, "repo");
        assert_eq!(p.fullname, "owner/repo");
        assert!(remote_url_is_valid(&p));
    }

    #[test]
    fn parse_https_with_user() {
        let p = parse_git_url("https://user@github.com/owner/repo").unwrap();
        assert_eq!(p.host.as_deref(), Some("github.com"));
        assert_eq!(p.owner.as_deref(), Some("owner"));
        assert_eq!(p.name, "repo");
        assert!(remote_url_is_valid(&p));
    }

    #[test]
    fn parse_ssh_with_user() {
        let p = parse_git_url("ssh://git@github.com/owner/repo").unwrap();
        assert_eq!(p.host.as_deref(), Some("github.com"));
        assert_eq!(p.owner.as_deref(), Some("owner"));
        assert_eq!(p.name, "repo");
        assert!(remote_url_is_valid(&p));
    }

    #[test]
    fn parse_ssh_without_user() {
        let p = parse_git_url("ssh://github.com/owner/repo.git").unwrap();
        assert_eq!(p.host.as_deref(), Some("github.com"));
        assert_eq!(p.owner.as_deref(), Some("owner"));
        assert_eq!(p.name, "repo");
        assert!(remote_url_is_valid(&p));
    }

    #[test]
    fn parse_git_scheme() {
        let p = parse_git_url("git://github.com/owner/repo.git").unwrap();
        assert_eq!(p.host.as_deref(), Some("github.com"));
        assert_eq!(p.owner.as_deref(), Some("owner"));
        assert_eq!(p.name, "repo");
        assert!(remote_url_is_valid(&p));
    }

    #[test]
    fn parse_scp_like() {
        let p = parse_git_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(p.host.as_deref(), Some("github.com"));
        assert_eq!(p.owner.as_deref(), Some("owner"));
        assert_eq!(p.name, "repo");
        assert!(remote_url_is_valid(&p));
    }

    #[test]
    fn parse_no_scheme() {
        let p = parse_git_url("github.com/owner/repo").unwrap();
        assert_eq!(p.host.as_deref(), Some("github.com"));
        assert_eq!(p.owner.as_deref(), Some("owner"));
        assert_eq!(p.name, "repo");
        assert!(remote_url_is_valid(&p));
    }

    #[test]
    fn invalid_empty_input() {
        assert!(parse_git_url("   ").is_none());
    }

    #[test]
    fn invalid_missing_repo_https() {
        assert!(parse_git_url("https://github.com/owner").is_none());
    }

    #[test]
    fn invalid_missing_repo_scp() {
        assert!(parse_git_url("git@github.com:owner").is_none());
    }

    #[test]
    fn invalid_no_path_ssh() {
        assert!(parse_git_url("ssh://user@github.com").is_none());
    }

    #[test]
    fn invalid_user_at_no_colon() {
        assert!(parse_git_url("user@host/path").is_none());
    }

    #[test]
    fn invalid_host_empty_in_scheme() {
        // e.g., https:///owner/repo -> host part empty, still parses but invalid by validation
        let p = parse_git_url("https:///owner/repo").unwrap();
        assert!(!remote_url_is_valid(&p));
        assert_eq!(p.host.as_deref(), Some(""));
        assert_eq!(p.owner.as_deref(), Some("owner"));
        assert_eq!(p.name, "repo");
    }

    #[test]
    fn remote_url_is_valid_checks() {
        let p = parse_git_url("https://github.com/owner/repo").unwrap();
        assert!(remote_url_is_valid(&p));

        let p2 = ParsedGitUrl {
            host: Some("".to_string()),
            owner: Some("owner".to_string()),
            name: "repo".to_string(),
            fullname: "owner/repo".to_string(),
        };
        assert!(!remote_url_is_valid(&p2));

        let p3 = ParsedGitUrl {
            host: Some("github.com".to_string()),
            owner: Some(" ".to_string()),
            name: "repo".to_string(),
            fullname: "owner/repo".to_string(),
        };
        assert!(!remote_url_is_valid(&p3));
    }

    #[test]
    fn parse_scp_owner_only_trailing_slash() {
        // Expect owner parsed, and repo name considered missing (empty string)
        let p = parse_git_url("git@github.com:owner/").unwrap();
        assert_eq!(p.host.as_deref(), Some("github.com"));
        assert_eq!(p.owner.as_deref(), Some("owner"));
        assert!(p.name.is_empty());
    }
}
