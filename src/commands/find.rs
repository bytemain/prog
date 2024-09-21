use crate::context::Context;

pub fn run(c: &Context, keyword: &String) {
    c.storage().find(keyword);
}
