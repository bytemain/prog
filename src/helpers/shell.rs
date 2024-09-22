use log::info;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};

pub(crate) fn run(cmd: String) -> anyhow::Result<()> {
    println!("Running {}", cmd);

    let child = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    let output = child
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;

    let reader = BufReader::new(output);

    reader.lines().filter_map(|line| line.ok()).for_each(|line| println!("{}", line));

    Ok(())
}
