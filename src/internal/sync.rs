use crate::context::Context;
use crate::helpers::git::get_remote_url;
use git_url_parse::GitUrl;
use ignore::WalkBuilder;
use log::{error, info};
use rayon::prelude::*;
use std::{sync::mpsc::channel, time::Instant};

#[derive(Debug, Clone)]
pub struct SyncItem {
    pub host: String,
    pub repo: String,
    pub owner: String,
    pub remote_url: String,
    pub base_dir: String,
    pub full_path: String,
}

fn read_repo_from_dir(dir: &str) -> Vec<SyncItem> {
    let mut repos: Vec<SyncItem> = Vec::new();
    let (tx, rx) = channel::<SyncItem>();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(12);

    WalkBuilder::new(dir)
        .threads(threads)
        .max_depth(Some(3))
        .hidden(true)
        .filter_entry(|entry| entry.file_type().unwrap().is_dir())
        .build_parallel()
        .run(|| {
            let tx_clone = tx.clone();
            Box::new(move |result_entry| {
                match result_entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let dot_git_path = path.join(".git");
                        // Check if .git is a directory (standard for git repos) or a file (for worktrees)
                        if !dot_git_path.exists() {
                            return ignore::WalkState::Continue;
                        }

                        let full_path_str = path.display().to_string();

                        let remote_url_str = get_remote_url(&full_path_str);
                        if remote_url_str.is_empty() {
                            log::warn!("Could not determine remote URL for git repository: {}. Skipping item.", full_path_str);
                            return ignore::WalkState::Continue;
                        }

                        match GitUrl::parse(&remote_url_str) {
                            Ok(url_parsed) => {
                                if url_parsed.host.is_none() || url_parsed.owner.is_none() || url_parsed.name.is_empty() {
                                    log::error!(
                                        "Invalid Git URL '{}' (missing host, owner, or repo name) for path: {}. Skipping item.",
                                        remote_url_str,
                                        full_path_str
                                    );
                                    return ignore::WalkState::Continue;
                                }

                                let host_name = url_parsed.host.unwrap();
                                let owner_name = url_parsed.owner.unwrap();
                                let repo_name = url_parsed.name;

                                let item = SyncItem {
                                    base_dir: dir.to_string(),
                                    host: host_name,
                                    repo: repo_name,
                                    owner: owner_name,
                                    remote_url: remote_url_str.clone(),
                                    full_path: full_path_str,
                                };

                                if let Err(e) = tx_clone.send(item) {
                                    error!("Failed to send SyncItem on channel: {}. Quitting walk.", e);
                                    return ignore::WalkState::Quit; // Critical error in channel communication.
                                }
                            }
                            Err(e) => {
                                log::error!(
                                    "Failed to parse remote URL '{}' for git repository at '{}': {}. Skipping item.",
                                    remote_url_str,
                                    full_path_str,
                                    e
                                );
                                return ignore::WalkState::Continue;
                            }
                        }
                    }
                    Err(err) => {
                        // Log errors for individual entries but continue the walk.
                        error!("Error processing entry during directory walk: {}", err);
                    }
                }
                ignore::WalkState::Continue
            })
        });

    // Drop the original sender. This is crucial for the receiver loop to terminate
    // once all worker threads have finished and dropped their sender clones.
    drop(tx);

    // Collect all SyncItems sent through the channel.
    // This loop will block until the channel is empty and all senders are dropped.
    for item in rx {
        repos.push(item);
    }

    repos
}

pub fn sync(c: &Context, silent: bool) {
    if !silent {
        println!("Deleting old database...");
    }
    let now = Instant::now();
    c.database_mut().reset();

    if !silent {
        println!("Syncing...");
    }

    let base_dirs = c.config().base_dirs();

    let repos: Vec<SyncItem> =
        base_dirs.par_iter().map(|base_dir| read_repo_from_dir(base_dir)).flatten().collect();

    for repo in repos {
        if !silent {
            println!("Syncing {:?}", repo.full_path);
        }
        c.database_mut().record_item(
            &repo.base_dir,
            &repo.remote_url,
            &repo.host,
            &repo.repo,
            &repo.owner,
            &repo.full_path,
        );
    }

    c.database_mut().update_last_sync_time();
    if let Err(e) = c.database_mut().save() {
        error!("Failed to save database: {}", e);
    }

    if !silent {
        println!("Synced");
        println!("Elapsed {}ms", now.elapsed().as_millis());
    }
}

pub fn check_auto_sync(c: &Context) {
    let interval = c.config().get_auto_sync_interval_secs();
    if interval <= 0 {
        return;
    }

    let last_sync_time = c.database().get_last_sync_time();
    if last_sync_time.is_none() {
        sync(c, true);
    } else {
        let now = chrono::Utc::now().naive_utc();
        let duration = now - last_sync_time.unwrap();
        if duration.num_seconds() > interval {
            sync(c, true);
        }
    }
}
