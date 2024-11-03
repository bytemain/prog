use crate::context::Context;
use inquire::Confirm;
use log::info;
use std::path::{Path, PathBuf};

pub fn run(c: &Context, path: PathBuf) {
    let path_str = shellexpand::tilde(&path.to_str().unwrap()).into_owned();
    info!("Remove: {:?}", path_str);

    let path = Path::new(&path_str);
    if !path.exists() {
        eprintln!("Path not found: {:?}", path);
        return;
    }

    let ans = Confirm::new("Are you sure you want to remove this repository?")
        .with_default(false)
        .with_help_message("Removed repository cannot be restored!")
        .prompt();

    match ans {
        Ok(true) => {
            std::fs::remove_dir_all(&path).expect("Error removing directory");
            c.database_mut().remove(&path_str);
            println!("Repository removed: {:?}", path);
        }
        _ => println!("\nCancelled"),
    }
}
