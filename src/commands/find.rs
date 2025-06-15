use crate::{
    cli::ECommands,
    context::Context,
    helpers::{path, platform},
};
use clap::builder::Str;
use inquire::Select;
use std::collections::HashSet;

use super::printer::error::handle_inquire_error;

fn handle_result(path: &str) {
    println!("Found: {}", path);
    platform::clipboard::copy_path(path);
}

pub fn find_keyword(c: &Context, keyword: &str) -> Option<Vec<String>> {
    c.auto_sync_silent();

    let result = c.database_mut().find(keyword);
    if result.is_empty() {
        return None;
    }

    let mut options = HashSet::<String>::new();

    let mut should_sync = false;
    for repo in result {
        let path: String = repo.full_path.clone();
        if path::exists(&path) {
            options.insert(path.clone());

            if repo.host == keyword {
                options.insert(repo.host_fs_path());
            }

            if repo.owner == keyword {
                options.insert(repo.owner_fs_path());
            }
        } else {
            should_sync = true;
        }
    }

    if should_sync {
        c.sync_silent();
    }

    Some(options.into_iter().collect())
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
        println!("{}", &result[0]);
        return;
    }

    let ans = Select::new("Which project are you looking for?", result).prompt();

    match ans {
        Ok(choice) => {
            println!("{}", &choice);
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

    let ans = Select::new("Which project are you looking for?", result).prompt();

    match ans {
        Ok(choice) => {
            handle_result(&choice);
            return true;
        }
        Err(e) => handle_inquire_error(e),
    }

    false
}
