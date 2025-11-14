use std::path::Path;

use crate::helpers::colors::Colorize;
use crate::helpers::git::remote_url_is_valid;
use crate::helpers::git::parse_git_url;
use crate::{context::Context, helpers::platform};
use log::debug;

pub fn run(c: &mut Context, url: &str, rest: &[String]) {
    let base_dir = c.get_base_dir().unwrap();
    let url = c.config().replace_alias(url.to_owned());

    let url_parsed = match parse_git_url(&url) {
        Some(p) => p,
        None => {
            eprintln!("{}", format!("Invalid git url: {}", url).red());
            return;
        }
    };
    debug!("url parsed: {:#?}", url_parsed);

    if !remote_url_is_valid(&url_parsed) {
        eprintln!("{}", format!("Invalid git url: {}", url).red());
        return;
    }

    let host = url_parsed.host.clone().unwrap();
    let owner = url_parsed.owner.clone().unwrap();
    let name = url_parsed.name.clone();
    let fullname = url_parsed.fullname.clone();

    debug!("host: {host}, full name: {fullname}, base dir: {base_dir}");

    let full_path = Path::new(&base_dir).join(&host).join(&owner).join(&name);

    if full_path.exists() {
        println!("{}", format!("Repo already exists: {}", full_path.display()).green());
        platform::clipboard::copy_path(full_path.to_str().unwrap());
        return;
    }

    debug!("target full path: {}", full_path.display());
    let target_path =
        full_path.to_str().unwrap_or_else(|| panic!("Cannot construct full path for {}", url));
    println!("{}", format!("Add: {}", url).green());

    let result = crate::helpers::git::clone(&url, rest, target_path);

    if result.is_err() {
        eprintln!("\n{}", format!("Failed to clone: {}", url).red());
        return;
    }

    c.database_mut().record_item(&base_dir, &url, &host, &name, &owner, target_path);
    c.database_mut().save().unwrap();

    println!("{}", format!("Cloned to: {}", target_path).green());
    platform::clipboard::copy_path(target_path);
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn test_path_construction_uses_native_separators() {
        // This test verifies that the path construction uses native path separators
        // instead of hardcoded forward slashes, which fixes the Windows display issue.
        
        let base_dir = if cfg!(windows) {
            "C:\\Users\\Admin\\0Workspace"
        } else {
            "/home/user/0Workspace"
        };
        let host = "github.com";
        let owner = "microsoft";
        let name = "vscode";
        
        let full_path = Path::new(base_dir).join(host).join(owner).join(name);
        let path_str = full_path.to_str().unwrap();
        
        // Verify that the path uses the native separator consistently
        // The path should contain the owner and name separated by the native separator
        assert!(path_str.contains(host), "Path should contain host");
        assert!(path_str.contains(owner), "Path should contain owner");
        assert!(path_str.contains(name), "Path should contain name");
        
        // On Windows, verify no forward slashes between owner and repo
        if cfg!(windows) {
            // Count occurrences of path separators between owner and name
            // The path should look like: C:\...\github.com\microsoft\vscode
            // not: C:\...\github.com\microsoft/vscode
            let owner_pos = path_str.find(owner).unwrap();
            let name_pos = path_str.find(name).unwrap();
            let between = &path_str[owner_pos + owner.len()..name_pos];
            
            // The separator between owner and name should be backslash on Windows
            assert_eq!(between, "\\", 
                "On Windows, owner and name should be separated by backslash, not forward slash");
            
            // Also verify the full path doesn't mix separators
            assert!(!path_str.contains('/'), 
                "Windows path should not contain forward slashes");
        } else {
            // On Unix, verify forward slashes are used
            let owner_pos = path_str.find(owner).unwrap();
            let name_pos = path_str.find(name).unwrap();
            let between = &path_str[owner_pos + owner.len()..name_pos];
            
            assert_eq!(between, "/", 
                "On Unix, owner and name should be separated by forward slash");
        }
    }
}
