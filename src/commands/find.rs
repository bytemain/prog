use std::fs;

use inquire::Select;
use log::debug;

use crate::{context::Context, helpers::{path, platform}};

fn handle_result(path: &str) {
    println!("Found: {}", path);
    platform::clipboard::copy_path(path);
}

pub fn find_keyword(c: &Context, keyword: &str) -> Option<Vec<String>> {
    let result = c.database_mut().find(keyword);
    if result.is_empty() {
        return None;
    }
    let mut  options = vec![];

    for repo in result {
        let path: String = repo.fs_path();
        if path::exists(&path) {
            debug!("exists {}", path.clone());
            options.push(path.clone());
        } else {
            c.database_mut().remove(path.as_str());
        }
    }

    Some(options)
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
        Err(_) => println!("There was an error, please try again"),
    }
}

pub fn check_keyword_exists(c: &Context, keyword: &str) -> bool {
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
        Err(_) => println!("There was an error, please try again"),
    }

    false
}
