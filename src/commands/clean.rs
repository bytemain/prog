use crate::context::Context;
use inquire::Confirm;
use log::error;

use super::printer::error::handle_inquire_error;

pub fn run(c: &Context, skip_confirmation: bool) {
    if !skip_confirmation {
        let ans = Confirm::new("You're cleaning all your repo records, continue?")
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

    c.database_mut().reset();
    println!("Successfully clean the database.");
    if let Err(e) = c.database_mut().save() {
        error!("Failed to save database: {}", e);
    }
}
