use inquire::Select;

use crate::{context::Context, helpers::platform};

fn handle_result(path: &str) {
    println!("Found: {}", path);
    platform::clipboard::copy_path(path);
}

pub fn find_keyword(c: &Context, keyword: &str) -> Option<Vec<String>> {
    let result = c.database_mut().find(keyword);
    if result.is_empty() {
        return None;
    }
    let options = result.iter().map(|r| r.fs_path()).collect::<Vec<_>>();
    Some(options)
}

pub fn run(c: &Context, keyword: &str) {
    println!("Searching for: {}", keyword);
    let result = find_keyword(c, keyword).unwrap_or_default();

    if result.is_empty() {
        println!("No result found");
        return;
    }

    if result.len() == 1 {
        let path = result[0].clone();
        handle_result(&path);
        return;
    }

    let ans = Select::new("Which project are you looking for?", result).prompt();

    match ans {
        Ok(choice) => {
            handle_result(&choice);
        }
        Err(_) => println!("There was an error, please try again"),
    }
}

pub fn run_from_not_matched(c: &Context, keyword: &str) -> bool {
    let result = find_keyword(c, keyword).unwrap_or_default();

    if result.is_empty() {
        return false;
    }

    if result.len() == 1 {
        let path = result[0].clone();
        handle_result(&path);
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
