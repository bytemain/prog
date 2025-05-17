use crate::constants;
use crate::context::configuration;
use crate::context::database;
use crate::helpers::path::PROGRAM;
use crate::helpers::colors::Colorize;
use log::debug;
use std::cell::OnceCell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::env;
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
        let config_file_path = constants::CONFIG_TOML_FILE.clone();
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

            let s = std::fs::read_to_string(&config_file_path).unwrap();
            let mut config: configuration::Config = toml::from_str(&s).unwrap();
            debug!("read config: {:?}", config);

            if config.tmp_dir.len() == 0 {
                let dir = env::temp_dir();
                config.tmp_dir = dir.to_string_lossy().to_string();
            }

            if config.base.len() == 0 {
                eprintln!(
                    "{}",
                    format!("No base path found, please add one to your config file: {}",
                    config_file_path.display()).red()
                );
                exit(1);
            }

            config
        })
    }
}
