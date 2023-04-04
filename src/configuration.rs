use indoc::indoc;
use serde::Deserialize;
use std::collections::HashMap;
#[derive(Deserialize, Debug)]

pub struct Config {
    #[serde(default)]
    pub base: Vec<String>,
    #[serde(default)]
    pub alias: HashMap<String, String>,
}

pub const DEFAULT_CONFIG_TOML: &str = indoc! {r#"
base = [

]

[alias]
"github://" = "https://github.com/"
# "gitlab://" = "https://gitlab.com/"
# "bitbucket://" = "https://bitbucket.org/"

"#
};
