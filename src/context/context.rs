use crate::constants;
use crate::context::configuration;
use crate::context::database;
use config::Config;
use log::{debug, error};
use std::cell::RefCell;
use std::cell::{LazyCell, Ref};
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

pub struct Context {
    pub config: RefCell<configuration::Config>,
    db: RefCell<LazyCell<database::Database, fn() -> database::Database>>,
}

impl Context {
    #[inline]
    pub fn new(config_file_path: PathBuf) -> Self {
        if !config_file_path.exists() {
            eprintln!(
                "Could not find config file at {}, create default",
                &config_file_path.display()
            );

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
            eprintln!(
                "No base path found, please add one to your config file: {}",
                config_file_path.display()
            );
            exit(1)
        }

        if !constants::DATABASE_FOLDER.exists() {
            match std::fs::create_dir_all(constants::DATABASE_FOLDER.clone()) {
                Ok(_) => {}
                Err(err) => panic!("Could not create database folder: {}", err),
            }
        }

        let db =
            RefCell::new(LazyCell::<database::Database, fn() -> database::Database>::new(|| {
                database::Database::new()
            }));
        let config = RefCell::new(config);

        Self { config, db }
    }

    #[inline]
    pub fn database(&self) -> Ref<'_, LazyCell<database::Database>> {
        self.db.borrow()
    }

    #[inline]
    pub fn config(&self) -> Ref<'_, configuration::Config> {
        self.config.borrow()
    }

    #[inline]
    pub fn delete_db_folder(&self) {
        let db_file = constants::DATABASE_FOLDER.clone();
        match std::fs::remove_dir_all(&db_file) {
            Ok(_) => {}
            Err(err) => error!("Could not delete db file: {}", err),
        }
    }
}
