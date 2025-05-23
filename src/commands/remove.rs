use log::error;

use crate::commands::printer::error::handle_inquire_error;
use crate::context::Context;
use crate::helpers::colors::Colorize;
use crate::helpers::path::remove_dir_with_empty_parents;
use inquire::Confirm;
use std::path::PathBuf;

pub fn run(c: &mut Context, path: PathBuf, skip_confirmation: bool) {
    let path_str = path.to_string_lossy();

    // If not skipping confirmation, prompt the user
    if !skip_confirmation {
        let ans = Confirm::new("You're removing a repo record, continue?")
            .with_default(false)
            .with_help_message("This won't delete your git repos in the disk")
            .prompt();

        let ans = match ans {
            Ok(true) => true,
            Ok(false) => {
                println!("Canceled.");
                false
            }
            Err(e) => {
                handle_inquire_error(e);
                false
            }
        };

        if !ans {
            return;
        }
    }

    // Get the base_dir from the database to use as stop_at parameter
    let base_dir_path = if let Some(repo) = c.database().get_by_path(&path_str) {
        let base_dir = repo.base_dir;
        Some(PathBuf::from(base_dir))
    } else {
        None
    };

    // Remove the directory and its empty parents, stopping at the base_dir
    remove_dir_with_empty_parents(&path, base_dir_path.as_ref())
        .expect("Error when removing directory");
    c.database_mut().remove(&path_str);

    println!("Repository removed: {:?}", path);

    if let Err(e) = c.database_mut().save() {
        error!("Failed to save database: {}", e);
    } else {
        println!("{}", "Done!".green());
    }
}
