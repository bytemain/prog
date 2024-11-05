use std::process::{Command, Stdio};

pub(crate) fn run(cmd: &str) -> anyhow::Result<()> {
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit())
        .spawn()
        .expect("Failed to execute command");

    child.wait().expect("command wasn't running");
    Ok(())
}
