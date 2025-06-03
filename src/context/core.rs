use crate::constants;
use crate::context::configuration;
use crate::context::database;
use crate::helpers::colors::Colorize;
use crate::helpers::path::get_config_path;
use crate::internal::sync::check_auto_sync;
use crate::internal::sync::sync;
use anyhow::bail;
use log::debug;
use std::cell::LazyCell;
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
    config_file_path: LazyCell<PathBuf>,
}

impl Context {
    #[inline]
    pub fn new() -> Self {
        let db = RefCell::new(database::Database::new());
        let config_file_path: LazyCell<PathBuf> =
            LazyCell::new(|| get_config_path(constants::CONFIG_TOML_FILE));
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

    pub fn get_base_dir(&self) -> anyhow::Result<String> {
        let base_dirs = self.config().base_dirs();
        if base_dirs.is_empty() {
            bail!("Please configure base dir in : {}", self.config_file_path.display());
        }

        if base_dirs.len() == 1 {
            return anyhow::Ok(base_dirs.first().unwrap().clone());
        }

        bail!("Not implemented multiple base dir yet");
    }

    fn init_config(&self) -> anyhow::Result<()> {
        let config_file_path = self.config_file_path.clone();

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

            if config.base.is_empty() {
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

    pub fn sync_silent(&self) {
        sync(self, true);
    }

    pub fn auto_sync_silent(&self) {
        check_auto_sync(self);
    }
}
