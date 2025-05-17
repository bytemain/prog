use crate::context::Context;
use crate::internal;

pub fn run(c: &Context) {
    internal::sync::sync(c, false);
}
