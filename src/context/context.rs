use crate::context::database;
use crate::{configuration, constants};
use config::Config;
use log::{debug, error};
use std::cell::Ref;
use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

pub struct Context {
    pub config: RefCell<configuration::Config>,
    storage: RefCell<database::Database>,
}

impl Context {
    #[inline]
    pub fn new(config_file_path: PathBuf) -> Self {
        if !config_file_path.exists() {
            error!("Could not find config file at {}, create default", &config_file_path.display());
            let config_dir = config_file_path.parent().unwrap();
            if !config_dir.exists() {
                match std::fs::create_dir_all(config_dir) {
                    Ok(_) => {}
                    Err(err) => panic!("Could not create config directory: {}", err),
                }
            }

            // auto create config file
            let mut config_file = match std::fs::File::create(&config_file_path) {
                Ok(file) => file,
                Err(err) => panic!("Could not create config file: {}", err),
            };
            if config_file_path.ends_with(".toml") {
                match config_file.write_all(constants::DEFAULT_CONFIG_TOML.as_bytes()) {
                    Ok(_) => {}
                    Err(err) => panic!("Could not write default config: {}", err),
                }
            }
        }

        let config_builder = Config::builder()
            .add_source(config::File::from(config_file_path.clone()))
            // Add in settings from the environment (with a prefix of PROG)
            // Eg.. `PROG_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(config::Environment::with_prefix("PROG"))
            .build()
            .unwrap();

        let config = config_builder.try_deserialize::<configuration::Config>().unwrap();
        debug!("read config: {:?}", config);

        if config.base.len() == 0 {
            error!(
                "No base path found, please add one to your config file: {}",
                config_file_path.display()
            );
            exit(1)
        }
        let storage = RefCell::new(database::Database::new());
        let config = RefCell::new(config);

        Self { config, storage }
    }

    #[inline]
    pub fn storage(&self) -> Ref<'_, database::Database> {
        self.storage.borrow()
    }

    #[inline]
    pub fn config(&self) -> Ref<'_, configuration::Config> {
        self.config.borrow()
    }
}
