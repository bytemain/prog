pub const DEFAULT_CONFIG_TOML: &str = r#"base = [
  "~/0Workspace"
]

auto_sync_interval_secs = 3600

[alias]
"github://" = "https://github.com/"
"gitlab://" = "https://gitlab.com/"
"bitbucket://" = "https://bitbucket.org/"
"#;

pub const DATABASE_FILE: &str = "data.toml";
pub const CONFIG_TOML_FILE: &str = "config.toml";
