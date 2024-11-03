use crate::context::Context;
use inquire::ui::RenderConfig;
use inquire::Confirm;
use crate::helpers::rand::gen_n_number;

pub fn run(c: &Context) {
    println!("This will delete all your repos in database, won't delete your git repos.");
    // generate random 4 number
    let random_number = gen_n_number(4);
    let ans = Confirm {
        message: &format!("You're doing a very dangerous operation, are you sure you want to continue?\nType {} or no to continue.", {&random_number}),
        starting_input: None,
        default: Some(false),
        placeholder: Some(&random_number),
        help_message: None,
        formatter: &|ans| match ans {
            true => String::from(&random_number),
            false => String::from("no"),
        },
        parser: &|ans| {
            if ans == &random_number {
                Ok(true)
            } else if ans == "no" {
                Ok(false)
            } else {
                Err(())
            }
        },
        error_message: format!("Type with {} or no", &random_number).into(),
        default_value_formatter: &|def| match def {
            true => String::from(&random_number),
            false => String::from("no"),
        },
        render_config: RenderConfig::default(),
    }
        .prompt()
        .unwrap();

    if ans {
        c.delete_db_folder();
        println!("Successfully clean the database.");
    } else {
        println!("Canceled.");
    }
}

