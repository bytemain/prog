use std::io::Write;

use crate::{constants, context::Context};

pub fn run(c: &mut Context) {
    let config_file_path = constants::CONFIG_TOML_FILE.clone();

    if config_file_path.exists() {
        println!("Config file already exists: {}", config_file_path.display());
        return;
    }

    let config_dir = config_file_path.parent().unwrap();
    if !config_dir.exists() {
        match std::fs::create_dir_all(config_dir) {
            Ok(_) => {}
            Err(err) => panic!("Could not create config directory: {}", err),
        }
    }

    let mut config_file = match std::fs::File::create(&config_file_path) {
        Ok(file) => file,
        Err(err) => panic!("Could not create config file: {}", err),
    };

    match config_file.write_all(constants::DEFAULT_CONFIG_TOML.as_bytes()) {
        Ok(_) => {}
        Err(err) => panic!("Could not write default config: {}", err),
    }
}
