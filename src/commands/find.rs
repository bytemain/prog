use crate::context::Context;

pub fn run(c: &Context, keyword: &str) {
    c.storage().find(keyword);
}
