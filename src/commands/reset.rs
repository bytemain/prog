use crate::context::Context;
use inquire::ui::RenderConfig;
use inquire::Confirm;
use rand::Rng;

pub fn run(c: &Context) {
    // generate random 4 number
    let random_number = rand::thread_rng().gen_range(1000..9999);
    let random_number_str: &str = &random_number.to_string();
    let ans = Confirm {
        message: &format!("You're doing a very dangerous operation, are you sure you want to continue?\nType '{}' or 'no' to continue.", {random_number}),
        starting_input: None,
        default: Some(false),
        placeholder: Some(&random_number.to_string()),
        help_message: None,
        formatter: &|ans| match ans {
            true => String::from(random_number_str),
            false => String::from("no"),
        },
        parser: &|ans| {
            if ans == random_number_str {
                Ok(true)
            } else if ans == "no" {
                Ok(false)
            } else {
                Err(())
            }
        },
        error_message: format!("Type with '{}' or 'no'", random_number).into(),
        default_value_formatter: &|def| match def {
            true => String::from(random_number_str),
            false => String::from("no"),
        },
        render_config: RenderConfig::default(),
    }
        .prompt()
        .unwrap();

    println!("Your answer: {ans}");
    if ans {
        c.delete_db_folder();
        println!("Reset done.");
    } else {
        println!("Reset canceled.");
    }
}
