use crate::context::Context;
use crate::context::database::models::Repo;
use crate::helpers::colors::Colorize;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Size constants
const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const GB: u64 = 1024 * MB;

// Threshold constants for color coding
const MB_100: u64 = 100 * MB;
const MB_500: u64 = 500 * MB;

/// Calculate the size of a directory recursively
fn calculate_dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    let mut total_size = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            // Check if the entry is a symlink using file_type() before following it
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_symlink() {
                    // Skip symlinks to avoid infinite loops
                    continue;
                }
                if file_type.is_dir() {
                    total_size += calculate_dir_size(&entry.path());
                } else if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }
    }
    total_size
}

/// Format bytes into a human-readable string (e.g., "1.5 GB", "256 MB")
fn format_size(bytes: u64) -> String {
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format size with color based on size thresholds
fn format_size_colored(bytes: u64) -> String {
    let size_str = format_size(bytes);

    if bytes >= GB {
        // > 1 GB - show in red
        format!("{}", size_str.red())
    } else if bytes >= MB_500 {
        // > 500 MB - show in yellow
        format!("{}", size_str.yellow())
    } else {
        format!("{}", size_str.green())
    }
}

/// Format size with color for subdirectories (different thresholds)
fn format_subdir_size_colored(bytes: u64) -> String {
    let size_str = format_size(bytes);

    if bytes >= GB {
        format!("{}", size_str.red())
    } else if bytes >= MB_100 {
        format!("{}", size_str.yellow())
    } else {
        size_str
    }
}

/// Struct to hold size information for a specific directory type
#[derive(Clone)]
struct DirSizeInfo {
    path: String,
    size: u64,
    dir_type: String,
}

/// Known large directories to look for
const KNOWN_LARGE_DIRS: &[(&str, &str)] = &[
    ("target", "Rust build artifacts"),
    ("node_modules", "Node.js dependencies"),
    (".git", "Git repository data"),
    ("vendor", "Vendored dependencies"),
    ("build", "Build output"),
    ("dist", "Distribution output"),
    ("out", "Output directory"),
    (".next", "Next.js build cache"),
    (".nuxt", "Nuxt.js build cache"),
    ("__pycache__", "Python bytecode cache"),
    (".venv", "Python virtual environment"),
    ("venv", "Python virtual environment"),
    ("Pods", "CocoaPods dependencies"),
    ("DerivedData", "Xcode build data"),
];

/// Check if a path contains any known large directories and return their info
fn find_large_dirs(repo_path: &Path) -> Vec<DirSizeInfo> {
    let mut found_dirs = Vec::new();

    for (dir_name, dir_type) in KNOWN_LARGE_DIRS {
        let dir_path = repo_path.join(dir_name);
        if dir_path.exists() && dir_path.is_dir() {
            let size = calculate_dir_size(&dir_path);
            if size > 0 {
                found_dirs.push(DirSizeInfo {
                    path: dir_path.to_string_lossy().to_string(),
                    size,
                    dir_type: dir_type.to_string(),
                });
            }
        }
    }

    found_dirs
}

/// Struct to hold repository size information
#[derive(Clone)]
struct RepoSizeInfo {
    repo: Repo,
    total_size: u64,
    large_dirs: Vec<DirSizeInfo>,
}

pub fn run(c: &mut Context) {
    c.auto_sync_silent();

    let items = c.database_mut().get_all_items();

    if items.is_empty() {
        println!("No repositories found. Use 'prog add <url>' to add a repository.");
        return;
    }

    println!("Calculating sizes for {} repositories...\n", items.len());

    // Calculate sizes in parallel
    let repo_sizes: Vec<RepoSizeInfo> = items
        .par_iter()
        .filter_map(|repo| {
            let repo_path = Path::new(&repo.full_path);
            if !repo_path.exists() {
                return None;
            }

            let total_size = calculate_dir_size(repo_path);
            let large_dirs = find_large_dirs(repo_path);

            Some(RepoSizeInfo {
                repo: repo.clone(),
                total_size,
                large_dirs,
            })
        })
        .collect();

    // Sort by total size (largest first)
    let mut sorted_repos = repo_sizes;
    sorted_repos.sort_by(|a, b| b.total_size.cmp(&a.total_size));

    // Calculate total size
    let grand_total: u64 = sorted_repos.iter().map(|r| r.total_size).sum();

    // Group by base_dir for organized output
    let mut grouped: HashMap<String, Vec<&RepoSizeInfo>> = HashMap::new();
    for repo_info in &sorted_repos {
        grouped.entry(repo_info.repo.base_dir.clone()).or_default().push(repo_info);
    }

    // Print results
    let mut base_dirs: Vec<_> = grouped.keys().cloned().collect();
    base_dirs.sort();

    for base_dir in base_dirs {
        if let Some(repos) = grouped.get(&base_dir) {
            let base_total: u64 = repos.iter().map(|r| r.total_size).sum();
            println!("{} ({})", base_dir.green(), format_size(base_total).blue());

            for repo_info in repos.iter() {
                let formatted_size = format_size_colored(repo_info.total_size);

                // Extract repo name from full path for cleaner display
                let display_path = repo_info
                    .repo
                    .full_path
                    .strip_prefix(&repo_info.repo.base_dir)
                    .unwrap_or(&repo_info.repo.full_path);
                let display_path = display_path.trim_start_matches('/');

                println!("  {:>10}  {}", formatted_size, display_path);

                // Show large directories within this repo
                if !repo_info.large_dirs.is_empty() {
                    for dir_info in &repo_info.large_dirs {
                        let dir_name = Path::new(&dir_info.path)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| dir_info.path.clone());

                        let colored_size = format_subdir_size_colored(dir_info.size);

                        println!(
                            "    {:>10}  {} ({})",
                            colored_size,
                            dir_name.blue(),
                            dir_info.dir_type
                        );
                    }
                }
            }
            println!();
        }
    }

    // Print summary
    println!("{}", "─".repeat(60));
    println!("Total: {}", format_size(grand_total).blue());
    println!("Repositories: {}", sorted_repos.len());

    // Print tips about external tools
    println!();
    println!("{}", "Recommended tools for disk space management:".green());
    println!("  {} - Interactive disk usage analyzer", "dust".blue());
    println!("  {} - Interactive disk usage analyzer", "ncdu".blue());
    println!("  {} - Interactive disk usage analyzer", "dua".blue());
    println!("  {} - Clean node_modules directories", "npkill".blue());
    println!("  {} - Clean Rust target directories", "cargo-sweep".blue());
    println!("  {} - Analyze Git repository size", "git-sizer".blue());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(10 * 1024), "10.00 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 + 512 * 1024), "1.50 MB");
        assert_eq!(format_size(100 * 1024 * 1024), "100.00 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 + 512 * 1024 * 1024), "1.50 GB");
        assert_eq!(format_size(70 * 1024 * 1024 * 1024), "70.00 GB");
    }

    #[test]
    fn test_calculate_dir_size_nonexistent() {
        let size = calculate_dir_size(Path::new("/nonexistent/path/12345"));
        assert_eq!(size, 0);
    }
}
