use std::process::{Command, Stdio};

use anyhow::bail;

pub(crate) fn run(cmd: &str) -> anyhow::Result<(), anyhow::Error> {
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit())
        .spawn()
        .expect("Failed to execute command");

    let status = child.wait().expect("command wasn't running");

    if !status.success() {
        bail!("Command failed with exit status: {}", status);
    }

    Ok(())
}
