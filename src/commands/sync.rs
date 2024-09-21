use crate::context::Context;
use std::fs::read_dir;
use crate::helpers::git::url::get_remote_url;


#[derive(Debug, Clone)]
pub struct SyncItem {
    pub host: String,
    pub repo: String,
    pub owner: String,
    pub full_path: String,
    pub remote_url: String,
}

fn read_repo_from_dir(dir: &str) -> Vec<SyncItem> {
    let mut repos = Vec::new();

    println!("Reading repos from {}", dir);
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
                            let full_path = repo.display().to_string();
                            let remote_url = get_remote_url(&full_path);
                            let item = SyncItem {
                                host: path.file_name().unwrap().to_string_lossy().to_string(),
                                repo: repo.file_name().unwrap().to_string_lossy().to_string(),
                                owner: owner.file_name().unwrap().to_string_lossy().to_string(),
                                full_path,
                                remote_url,
                            };

                            println!("{:#?}", item);
                            repos.push(item);
                        }
                    }
                }
            }
        }
    }
    repos
}

pub fn run(c: &Context) {
    println!("Syncing...");
    let base_dirs = c.path().get_all_base_dir();

    for base_dir in base_dirs {
        let repos = read_repo_from_dir(&base_dir);
        for repo in repos {
            println!("Syncing {:?}", repo);
            c.storage().record_item(&base_dir, &repo.remote_url, &repo.host, &repo.repo, &repo.owner);
        }
    }

    println!("Synced");
}