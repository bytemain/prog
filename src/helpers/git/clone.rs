use std::process::{Command, Stdio};
use anyhow::bail;

pub fn clone(
    url: &String,
    rest: &[String],
    target_path: &str,
) -> anyhow::Result<(), anyhow::Error> {
    let mut cmd = Command::new("git");
    cmd.arg("clone")
        .arg(url)
        .arg(target_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    
    // Add any additional arguments
    for arg in rest {
        cmd.arg(arg);
    }

    let status = cmd.status()?;

    if !status.success() {
        bail!("git clone failed with exit status: {}", status);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_clone_with_complex_path() {
        // This test validates that paths with spaces and special characters
        // are handled correctly by the clone function.
        
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a simple git repository to clone from
        let source_repo = temp_path.join("source-repo");
        fs::create_dir_all(&source_repo).unwrap();
        
        // Initialize a git repo in source
        Command::new("git")
            .args(&["init"])
            .current_dir(&source_repo)
            .output()
            .expect("Failed to initialize git repo");
        
        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(&source_repo)
            .output()
            .expect("Failed to configure git");
            
        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&source_repo)
            .output()
            .expect("Failed to configure git");
        
        // Create a test file and commit
        let test_file = source_repo.join("test.txt");
        fs::write(&test_file, "test content").unwrap();
        
        Command::new("git")
            .args(&["add", "."])
            .current_dir(&source_repo)
            .output()
            .expect("Failed to add files");
        
        Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&source_repo)
            .output()
            .expect("Failed to commit");
        
        // Create a target directory with spaces to simulate complex paths
        let complex_path = temp_path.join("test dir with spaces");
        fs::create_dir_all(&complex_path).unwrap();
        
        let target_path = complex_path.join("cloned-repo");
        let target_path_str = target_path.to_str().unwrap();
        
        // Clone the local repository using file:// URL
        let source_url = format!("file://{}", source_repo.display());
        let result = clone(&source_url, &[], target_path_str);
        
        // The clone should succeed
        assert!(result.is_ok(), "Clone failed: {:?}", result);
        
        // Verify the repository was cloned
        assert!(target_path.exists(), "Target path doesn't exist");
        assert!(target_path.join(".git").exists(), ".git directory doesn't exist");
        assert!(target_path.join("test.txt").exists(), "test.txt was not cloned");
    }

    #[test]
    fn test_clone_with_extra_args() {
        // Test that additional arguments are properly passed through
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a simple git repository
        let source_repo = temp_path.join("source-repo");
        fs::create_dir_all(&source_repo).unwrap();
        
        Command::new("git")
            .args(&["init"])
            .current_dir(&source_repo)
            .output()
            .expect("Failed to initialize git repo");
        
        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(&source_repo)
            .output()
            .unwrap();
            
        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&source_repo)
            .output()
            .unwrap();
        
        let test_file = source_repo.join("test.txt");
        fs::write(&test_file, "test").unwrap();
        
        Command::new("git")
            .args(&["add", "."])
            .current_dir(&source_repo)
            .output()
            .unwrap();
        
        Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&source_repo)
            .output()
            .unwrap();
        
        let target_path = temp_path.join("cloned-with-args");
        let target_path_str = target_path.to_str().unwrap();
        
        let source_url = format!("file://{}", source_repo.display());
        let extra_args = vec![String::from("--depth"), String::from("1")];
        
        let result = clone(&source_url, &extra_args, target_path_str);
        
        // Should succeed with the extra arguments
        assert!(result.is_ok(), "Clone with args failed: {:?}", result);
        assert!(target_path.exists());
    }
}
