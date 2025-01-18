use crate::context::Context;
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
            for entry in entries {
                if let Ok(entry) = entry {
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
                }
            }
        }
        Err(e) => eprintln!("Failed to read directory {}: {}", dir, e),
    }
}

pub fn run(c: &mut Context) {
    let temp = c.config().create_tmp_dir();
    println!("{}", temp);
}

pub fn clean_by_created(c: &mut Context) {
    let tmp_dir = c.config().get_tmp_dir();
    clean_directory(
        &tmp_dir,
        Duration::new(10 * 24 * 60 * 60, 0),
        |metadata| metadata.is_dir(),
        |metadata| metadata.created().or_else(|_| metadata.modified()),
    );
}

pub fn cleanoutdate(c: &mut Context) {
    let tmp_dir = c.config().get_tmp_dir();
    clean_directory(
        &tmp_dir,
        Duration::new(7 * 24 * 60 * 60, 0),
        |metadata| metadata.is_dir(),
        |metadata| metadata.modified(),
    );
}

pub fn list_files(c: &mut Context) {
    let tmp_dir = c.config().get_tmp_dir();
    let now = SystemTime::now();
    let seven_days_ago = now - Duration::from_secs(7 * 24 * 60 * 60);

    match fs::read_dir(&tmp_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        let created = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);
                        let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        let is_outdated = modified < seven_days_ago;
                        let outdated_marker = if is_outdated {
                            crossterm::style::Stylize::red(String::from("[outdated]"))
                        } else {
                            // 将modified 转换为 n days ago
                            let duration = now.duration_since(modified).unwrap();
                            let days = duration.as_secs() / (24 * 60 * 60);

                            if days > 0 {
                                crossterm::style::Stylize::green(format!("[{} days ago]", days))
                            } else {
                                crossterm::style::Stylize::green(format!(
                                    "[{} hours ago]",
                                    duration.as_secs() / (60 * 60)
                                ))
                            }
                        };

                        println!(
                            "{} {} Created: {} Modified: {}",
                            outdated_marker,
                            entry.path().to_string_lossy(),
                            chrono::DateTime::<chrono::Local>::from(created)
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string(),
                            chrono::DateTime::<chrono::Local>::from(modified)
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string()
                        );
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to read directory {}: {}", tmp_dir, e),
    }
}
