use crate::context::Context;
use crate::helpers::colors::Colorize;
use std::io::{self, Write};
use std::path::PathBuf;

pub fn run(c: &mut Context, path: PathBuf, skip_confirmation: bool) {
    let path_str = path.to_string_lossy();

    // If not skipping confirmation, prompt the user
    if !skip_confirmation {
        print!(
            "{} {} {}",
            "Are you sure you want to remove".yellow(),
            path_str.blue(),
            "[y/N]:".yellow()
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled.");
            return;
        }
    }

    println!("{}", format!("Removing: {}", path_str).yellow());
    c.database_mut().remove(&path_str);
    println!("{}", "Done!".green());
}
