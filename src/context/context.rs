use crate::helpers::path::PROGRAM;
use crate::constants;
use crate::context::configuration;
use crate::context::database;
use config::Config;
use log::{debug, error};
use std::cell::OnceCell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::path::PathBuf;
use std::process::exit;

pub struct Context {
    pub config: OnceCell<configuration::Config>,
    db: RefCell<database::Database>,
    config_file_path: PathBuf,
}

impl Context {
    #[inline]
    pub fn new() -> Self {
        let db = RefCell::new(database::Database::new());
        let mut config_file_path = constants::CONFIG_TOML_FILE.clone();
        let config: OnceCell<configuration::Config> = OnceCell::new();

        Self { config, db, config_file_path }
    }

    #[inline]
    pub fn database_mut(&self) -> RefMut<'_, database::Database> {
        self.db.borrow_mut()
    }

    #[inline]
    pub fn config(&self) -> &configuration::Config {
        self.config.get_or_init(|| {
            let config_file_path = self.config_file_path.clone();
            if !config_file_path.exists() {
                eprintln!(
                    "Could not find config file at {}, you can create one by running `{PROGRAM} init`",
                    &config_file_path.display()
                );
                exit(1);
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
                exit(1);
            }

            if !constants::DATABASE_FOLDER.exists() {
                match std::fs::create_dir_all(constants::DATABASE_FOLDER.clone()) {
                    Ok(_) => {}
                    Err(err) => panic!("Could not create database folder: {}", err),
                }
            }

            config
        })
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
