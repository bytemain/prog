use crate::context::Context;

pub fn run(c: &mut Context) {
    c.auto_sync_silent();

    let items = c.database_mut().get_all_items();
    for item in items {
        println!("{:}", item.full_path);
    }
}
