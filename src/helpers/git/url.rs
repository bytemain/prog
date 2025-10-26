use std::process::Command;

use git_url_parse::GitUrl;

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

pub fn remote_url_is_valid(parsed: &GitUrl) -> bool {
    // Validate remote URL: host and owner must exist and be non-empty
    parsed.host.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false)
        && parsed.owner.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false)
}
