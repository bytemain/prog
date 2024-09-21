use std::process::Command;

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
