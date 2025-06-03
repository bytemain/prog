use crate::context::Context;
use crate::helpers::colors::Colorize;
use crate::helpers::path::ensure_dir_exists;
use crate::helpers::platform;
use std::fs;
use std::time::{Duration, SystemTime};

fn clean_directory<F, G>(dir: &str, duration: Duration, is_target: F, get_time: G)
where
    F: Fn(&fs::Metadata) -> bool,
    G: Fn(&fs::Metadata) -> std::io::Result<SystemTime>,
{
    let now = SystemTime::now();
    let threshold = now - duration;

    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                println!("Check {}", entry.path().display());
                if let Ok(metadata) = entry.metadata() {
                    if is_target(&metadata) {
                        if let Ok(time) = get_time(&metadata) {
                            if time < threshold {
                                if metadata.is_file() {
                                    if let Err(e) = fs::remove_file(entry.path()) {
                                        eprintln!(
                                            "Failed to delete file {:?}: {}",
                                            entry.path(),
                                            e
                                        );
                                    } else {
                                        println!("Deleted file {:?}", entry.path());
                                    }
                                } else if metadata.is_dir() {
                                    if let Err(e) = fs::remove_dir_all(entry.path()) {
                                        eprintln!(
                                            "Failed to delete directory {:?}: {}",
                                            entry.path(),
                                            e
                                        );
                                    } else {
                                        println!("Deleted directory {:?}", entry.path());
                                    }
                                }
                            }
                        }
                    }
                }

                let target = entry.path();
                if target.is_dir() {
                    // 检查目录是否为空，如果为空则删除
                    if fs::read_dir(&target).unwrap().next().is_none() {
                        if let Err(e) = fs::remove_dir_all(&target) {
                            eprintln!(
                                "Failed to delete empty directory {}: {}",
                                &target.to_str().unwrap_or("N/A"),
                                e
                            );
                        } else {
                            println!(
                                "Deleted empty directory {}",
                                &target.to_str().unwrap_or("N/A")
                            );
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to read directory {}: {}", dir, e),
    }
}

pub fn run(c: &mut Context) {
    let path = c.config().create_tmp_dir();
    let path_str = path.to_string_lossy();
    println!("{}", path.display());
    ensure_dir_exists(&path);
    platform::clipboard::copy_path(&path_str);
}

pub fn cleanoutdate(c: &mut Context) {
    let tmp_dir = c.config().tmp_dir();
    clean_directory(
        &tmp_dir,
        Duration::new(7 * 24 * 60 * 60, 0),
        |metadata| metadata.is_dir(),
        |metadata| metadata.modified(),
    );
}

pub fn list_files(c: &mut Context) {
    let tmp_dir = c.config().tmp_dir();
    let now = SystemTime::now();
    let seven_days_ago = now - Duration::from_secs(7 * 24 * 60 * 60);

    match fs::read_dir(&tmp_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let created = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);
                    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    let is_outdated = modified < seven_days_ago;
                    let outdated_marker = if is_outdated {
                        String::from("[outdated]").red()
                    } else {
                        // 将modified 转换为 n days ago
                        let duration = now.duration_since(modified).unwrap();
                        let days = duration.as_secs() / (24 * 60 * 60);

                        if days > 0 {
                            format!("[{} days ago]", days).green()
                        } else {
                            format!("[{} hours ago]", duration.as_secs() / (60 * 60)).green()
                        }
                    };

                    println!(
                        "{} {} Created: {} Modified: {}",
                        outdated_marker,
                        entry.path().to_string_lossy(),
                        chrono::DateTime::<chrono::Local>::from(created)
                            .format("%Y-%m-%d %H:%M:%S"),
                        chrono::DateTime::<chrono::Local>::from(modified)
                            .format("%Y-%m-%d %H:%M:%S")
                    );
                }
            }
        }
        Err(e) => eprintln!("Failed to read directory {}: {}", tmp_dir, e),
    }
}
