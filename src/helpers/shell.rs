use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};

fn run(cmd: String) -> anyhow::Result<()> {
    println!("[RUN] {}", cmd);

    let child = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .spawn()
        .expect("git clone command failed to start");

    let output = child
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;

    let reader = BufReader::new(output);

    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));

    Ok(())
}

// 复杂命令，直接扔进bash执行
pub fn clone(url: &String, rest: &Vec<String>, target_path: &str) -> anyhow::Result<()> {
    let mut list = vec!["git", "clone", url, target_path];
    let mut rest_str: Vec<&str> = rest.iter().map(|x| x.as_str()).collect();
    list.append(&mut rest_str);

    run(list.join(" "))
}
