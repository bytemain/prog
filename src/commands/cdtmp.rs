use crate::context::Context;

pub fn run(c: &mut Context) {
    let temp = c.config().create_tmp_dir();
    println!("{}", temp);
}
