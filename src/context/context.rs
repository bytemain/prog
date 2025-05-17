use crate::constants;
use crate::context::configuration;
use crate::context::database;
use crate::helpers::colors::Colorize;
use crate::helpers::path::PROGRAM;
use crate::internal::sync::check_auto_sync;
use crate::internal::sync::sync;
use anyhow::bail;
use log::debug;
use std::cell::OnceCell;
use std::cell::RefCell;
use std::cell::{Ref, RefMut};
use std::env;
use std::io::Write;
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
    pub fn database(&self) -> Ref<'_, database::Database> {
        self.db.borrow()
    }

    #[inline]
    pub fn database_mut(&self) -> RefMut<'_, database::Database> {
        self.db.borrow_mut()
    }

    fn init_config(&self) -> anyhow::Result<()> {
        let config_file_path = constants::CONFIG_TOML_FILE.clone();

        if config_file_path.exists() {
            return Ok(());
        }

        let config_dir = config_file_path.parent().unwrap();
        if !config_dir.exists() {
            match std::fs::create_dir_all(config_dir) {
                Ok(_) => {}
                Err(err) => bail!("Could not create config directory: {}", err),
            }
        }

        let mut config_file = match std::fs::File::create(&config_file_path) {
            Ok(file) => file,
            Err(err) => bail!("Could not create config file: {}", err),
        };

        match config_file.write_all(constants::DEFAULT_CONFIG_TOML.as_bytes()) {
            Ok(_) => Ok(()),
            Err(err) => bail!("Could not write default config: {}", err),
        }
    }

    #[inline]
    pub fn config(&self) -> &configuration::Config {
        self.config.get_or_init(|| {
            let config_file_path = self.config_file_path.clone();
            if !config_file_path.exists() {
                let result = self.init_config();
                if result.is_err() {
                    eprintln!(
                        "{}",
                        format!("Could not initialize config file: {}", config_file_path.display())
                            .red()
                    );
                    eprintln!("{}", result.unwrap_err());
                    exit(1);
                }
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
                    format!(
                        "No base path found, please add one to your config file: {}",
                        config_file_path.display()
                    )
                    .red()
                );
                exit(1);
            }

            config
        })
    }

    pub fn sync(&self) {
        sync(self, true);
    }

    pub fn auto_sync(&self) {
        check_auto_sync(self);
    }
}
