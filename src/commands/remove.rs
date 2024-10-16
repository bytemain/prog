use crate::context::Context;
use log::info;
use std::path::PathBuf;

pub fn run(c: &Context, path: PathBuf) {
    let path = shellexpand::tilde(&path.to_str().unwrap()).into_owned();
    info!("Remove: {:?}", path);
    c.database_mut().remove(&path);
}
