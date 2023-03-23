mod configuration;

use config::{Config, FileFormat};

use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};

fn join_home_dir(path: &str) -> String {
    let mut home_dir = match env::var_os("HOME") {
        Some(path) => PathBuf::from(path),
        None => panic!("Could not find home directory"),
    };

    home_dir.push(path);

    match home_dir.to_str() {
        Some(path_str) => path_str.to_owned(),
        None => String::new(),
    }
}

fn main() {
    let config_file_path_string = join_home_dir(".prog/config.toml");
    let config_path = Path::new(&config_file_path_string);
    if !config_path.exists() {
        print!(
            "Could not find config file at {}, create default",
            config_file_path_string
        );
        let config_dir = config_path.parent().unwrap();
        if !config_dir.exists() {
            match std::fs::create_dir_all(config_dir) {
                Ok(_) => {}
                Err(err) => panic!("Could not create config directory: {}", err),
            }
        }

        // create config file
        let mut config_file = match std::fs::File::create(&config_file_path_string) {
            Ok(file) => file,
            Err(err) => panic!("Could not create config file: {}", err),
        };

        match config_file.write_all(configuration::DEFAULT_CONFIG.as_bytes()) {
            Ok(_) => {}
            Err(err) => panic!("Could not write default config: {}", err),
        }
    }

    println!("Final Path: {:?}", config_file_path_string);
    let settings = Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::new(
            config_file_path_string.as_str(),
            FileFormat::Toml,
        ))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("PROG"))
        .build()
        .unwrap();

    // Print out our settings (as a HashMap)
    println!(
        "{:?}",
        settings.try_deserialize::<configuration::Config>().unwrap()
    );
}
