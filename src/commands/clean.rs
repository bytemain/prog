use crate::context::Context;
use inquire::Confirm;

pub fn run(c: &Context, skip_confirmation: bool) {
    if !skip_confirmation {
        let ans = Confirm::new("You're cleaning all your repo records, continue?")
            .with_default(false)
            .with_help_message("This won't delete your git repos in the disk")
            .prompt();

        match ans {
            Ok(true) => {
                c.delete_db_folder();
                println!("Successfully clean the database.");
            }
            Ok(false) => {
                println!("Canceled.");
            }
            Err(_) => println!("Error with questionnaire, try again later"),
        }
    } else {
        c.delete_db_folder();
        println!("Successfully clean the database.");
    }
}
