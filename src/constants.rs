use std::{cell::LazyCell, path::PathBuf};

use crate::helpers::path::get_config_path;

pub const DEFAULT_CONFIG_TOML: &str = r#"base = [
  "~/0Workspace"
]

auto_sync_interval_secs = 3600

[alias]
"github://" = "https://github.com/"
"gitlab://" = "https://gitlab.com/"
"bitbucket://" = "https://bitbucket.org/"
"#;

pub const CONFIG_TOML_FILE: LazyCell<PathBuf> = LazyCell::new(|| get_config_path("config.toml"));

pub const DATABASE_FOLDER: LazyCell<PathBuf> = LazyCell::new(|| get_config_path("data"));
