use inquire::Select;

use crate::{context::Context, helpers::platform};

fn handle_result(path: &str) {
    println!("Found: {}", path);
    platform::clipboard::copy_path(path);
}

pub fn run(c: &Context, keyword: &str) {
    let result = c.database().find(keyword);

    if result.is_empty() {
        println!("No result found");
        return;
    }

    if result.len() == 1 {
        let path = result[0].fs_path();
        handle_result(&path);
        return;
    }

    let options = result.iter().map(|r| r.fs_path()).collect::<Vec<_>>();

    let ans = Select::new("Which project are you looking for?", options).prompt();

    match ans {
        Ok(choice) => {
            handle_result(&choice);
        }
        Err(_) => println!("There was an error, please try again"),
    }
}
