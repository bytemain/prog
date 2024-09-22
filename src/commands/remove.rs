use crate::context::Context;
use crate::helpers::path::expand_path;
use log::info;
use std::path::PathBuf;

pub fn run(c: &Context, path: PathBuf) {
    let path = shellexpand::tilde(&path.to_str().unwrap()).into_owned();
    info!("Remove: {:?}", path);
    c.storage().remove(&path);
}
