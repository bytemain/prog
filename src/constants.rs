use indoc::indoc;

pub const DEFAULT_CONFIG_TOML: &str = indoc! {r#"
base = [

]

[alias]
"github://" = "https://github.com/"
# "gitlab://" = "https://gitlab.com/"
# "bitbucket://" = "https://bitbucket.org/"

"#
};

pub const DEFAULT_CONFIG_PATH: &str = ".prog";

pub const CONFIG_TOML_FILE: &str = "config.toml";

pub const DATABASE_FILE: &str = "database.toml";
