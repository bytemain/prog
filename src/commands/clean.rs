use crate::context::Context;
use inquire::Confirm;

use super::printer::error::handle_inquire_error;

pub fn run(c: &Context, skip_confirmation: bool) {
    if !skip_confirmation {
        let ans = Confirm::new("You're cleaning all your repo records, continue?")
            .with_default(false)
            .with_help_message("This won't delete your git repos in the disk")
            .prompt();

        match ans {
            Ok(true) => {
                c.database_mut().clear();
                println!("Successfully clean the database.");
            }
            Ok(false) => {
                println!("Canceled.");
            }
            Err(e) => handle_inquire_error(e),
        }
    } else {
        c.database_mut().clear();
        println!("Successfully clean the database.");
    }
}
