use serde::Deserialize;
use std::collections::HashMap;
#[derive(Deserialize, Debug)]

pub struct Config {
    #[serde(default)]
    pub base: Vec<String>,
    #[serde(default)]
    pub alias: HashMap<String, String>,
}
