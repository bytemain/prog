use crate::{context::Context, helpers::platform};

pub fn run(c: &Context, keyword: &str) {
    let result = c.storage().find(keyword);

    if result.is_empty() {
        println!("No result found");
        return;
    }

    if result.len() == 1 {
        let path = result[0].fs_path();
        println!("Found {}", path);
        platform::clipboard::copy_path(&path);
        return;
    }
}
