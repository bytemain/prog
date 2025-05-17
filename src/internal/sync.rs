use crate::context::Context;
use crate::helpers::git::get_remote_url;
use log::{error, info};
use std::fs::read_dir;

#[derive(Debug, Clone)]
pub struct SyncItem {
    pub host: String,
    pub repo: String,
    pub owner: String,
    pub remote_url: String,
    pub full_path: String,
}

fn read_repo_from_dir(dir: &str) -> Vec<SyncItem> {
    let mut repos = Vec::new();

    info!("Reading repos from {}", dir);
    let paths = std::fs::read_dir(dir).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() {
            let owners = read_dir(&path).unwrap();
            for owner in owners {
                let owner = owner.unwrap().path();
                if owner.is_dir() {
                    let repos_in_origin = read_dir(&owner).unwrap();
                    for repo in repos_in_origin {
                        let repo = repo.unwrap().path();
                        if repo.is_dir() {
                            // check if it's a git repo
                            let git_dir = repo.join(".git");
                            if !git_dir.exists() {
                                continue;
                            }

                            let full_path = repo.display().to_string();
                            let remote_url = get_remote_url(&full_path);
                            let item = SyncItem {
                                host: path.file_name().unwrap().to_string_lossy().to_string(),
                                repo: repo.file_name().unwrap().to_string_lossy().to_string(),
                                owner: owner.file_name().unwrap().to_string_lossy().to_string(),
                                remote_url,
                                full_path,
                            };

                            info!("{:#?}", item);
                            repos.push(item);
                        }
                    }
                }
            }
        }
    }
    repos
}

pub fn sync(c: &Context, silent: bool) {
    if !silent {
        info!("Deleting old database...");
    }
    c.database_mut().reset();

    if !silent {
        info!("Syncing...");
    }
    let base_dirs = c.config().get_all_base_dir();

    for base_dir in base_dirs {
        let repos = read_repo_from_dir(&base_dir);
        for repo in repos {
            if !silent {
                println!("Syncing {:?}", repo.full_path);
            }
            c.database_mut().record_item(
                &base_dir,
                &repo.remote_url,
                &repo.host,
                &repo.repo,
                &repo.owner,
                &repo.full_path,
            );
        }
    }
    c.database_mut().update_last_sync_time();
    if let Err(e) = c.database_mut().save() {
        error!("Failed to save database: {}", e);
    }

    if !silent {
        info!("Synced");
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
