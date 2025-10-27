use crate::{
    context::Context,
    helpers::{git, path, platform},
};
use inquire::Select;

use super::printer::error::handle_inquire_error;

use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Debug)]
pub struct FoundItem {
    pub file_path: String,
    pub branch: String,
}

impl Display for FoundItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.branch.trim().is_empty() {
            write!(f, "{}", self.file_path)
        } else {
            write!(f, "{} [branch:{}]", self.file_path, self.branch)
        }
    }
}

fn handle_result(item: &FoundItem) {
    if item.branch.trim().is_empty() {
        println!("Found: {}", item.file_path);
    } else {
        println!("Found: {} [branch:{}]", item.file_path, item.branch);
    }
    platform::clipboard::copy_path(&item.file_path);
}

fn print_found_item_path(item: &FoundItem) {
    println!("{}", item.file_path);
}

pub fn find_keyword(c: &Context, keyword: &str) -> Option<Vec<FoundItem>> {
    c.auto_sync_silent();

    let result = c.database_mut().find(keyword);
    if result.is_empty() {
        return None;
    }

    let mut options = HashMap::<String, FoundItem>::new();

    let mut should_sync = false;
    for repo in result {
        let path_str: String = repo.full_path.clone();
        if path::exists(&path_str) {
            // Repo path entry with branch
            options.entry(path_str.clone()).or_insert_with(|| FoundItem {
                file_path: path_str.clone(),
                branch: git::get_branch(&path_str),
            });

            // Host directory entry (no branch)
            if repo.host == keyword {
                let host_path = repo.host_fs_path();
                options
                    .entry(host_path.clone())
                    .or_insert_with(|| FoundItem { file_path: host_path, branch: String::new() });
            }

            // Owner directory entry (no branch)
            if repo.owner == keyword {
                let owner_path = repo.owner_fs_path();
                options
                    .entry(owner_path.clone())
                    .or_insert_with(|| FoundItem { file_path: owner_path, branch: String::new() });
            }
        } else {
            should_sync = true;
        }
    }

    if should_sync {
        c.sync_silent();
    }

    Some(options.into_values().collect())
}

pub fn run(c: &Context, keyword: &str, _query: bool) {
    if _query {
        query(&c, &keyword);
    } else {
        find(&c, &keyword);
    }
}

pub fn query(c: &Context, keyword: &str) {
    let result = find_keyword(c, keyword).unwrap_or_default();

    if result.is_empty() {
        return;
    }

    if result.len() == 1 {
        print_found_item_path(&result[0]);
        return;
    }

    let ans = Select::new("Which project are you looking for?", result.clone()).prompt();

    match ans {
        Ok(choice) => {
            print_found_item_path(&choice);
        }
        Err(e) => handle_inquire_error(e),
    }
}

pub fn find(c: &Context, keyword: &str) -> bool {
    let result = find_keyword(c, keyword).unwrap_or_default();

    if result.is_empty() {
        return false;
    }

    if result.len() == 1 {
        handle_result(&result[0]);
        return true;
    }

    let ans = Select::new("Which project are you looking for?", result.clone()).prompt();

    match ans {
        Ok(choice) => {
            handle_result(&choice);
            return true;
        }
        Err(e) => handle_inquire_error(e),
    }

    false
}
