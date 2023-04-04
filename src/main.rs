mod configuration;

use config::Config;

use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::{Args, Parser, Subcommand};

#[derive(Subcommand, Debug)]
enum Commands {
    Clone { url: String, rest: Vec<String> },
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn join_home_dir(path: &str) -> String {
    let mut home_dir = match env::var_os("HOME") {
        Some(path) => PathBuf::from(path),
        None => panic!("Could not find home directory"),
    };

    home_dir.push(path);

    match home_dir.to_str() {
        Some(path_str) => path_str.to_owned(),
        None => String::new(),
    }
}

fn main() {
    let cli = Cli::parse();
    let mut config_file_path_string = join_home_dir(".prog/config.toml");

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
        config_file_path_string = config_path.to_str().unwrap().to_owned();
    }

    let config_path = Path::new(&config_file_path_string);
    if !config_path.exists() {
        print!(
            "Could not find config file at {}, create default",
            config_file_path_string
        );
        let config_dir = config_path.parent().unwrap();
        if !config_dir.exists() {
            match std::fs::create_dir_all(config_dir) {
                Ok(_) => {}
                Err(err) => panic!("Could not create config directory: {}", err),
            }
        }

        // auto create config file
        let mut config_file = match std::fs::File::create(&config_file_path_string) {
            Ok(file) => file,
            Err(err) => panic!("Could not create config file: {}", err),
        };
        if config_file_path_string.ends_with(".toml") {
            match config_file.write_all(configuration::DEFAULT_CONFIG_TOML.as_bytes()) {
                Ok(_) => {}
                Err(err) => panic!("Could not write default config: {}", err),
            }
        }
    }

    println!("Final Path: {:?}", config_file_path_string);
    let config_builder = Config::builder()
        .add_source(config::File::with_name(config_file_path_string.as_str()))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("PROG"))
        .build()
        .unwrap();

    let settings = config_builder
        .try_deserialize::<configuration::Config>()
        .unwrap();
    println!("{:?}", settings);

    if settings.base.len() == 0 {
        println!(
            "No base path found, please add one to your config file: {}",
            config_file_path_string
        );
        exit(1)
    }

    let mut not_match = false;
    match &cli.command {
        Some(Commands::Clone { url: repo, rest }) => {
            println!("Clone command given");
            println!("Repo: {}", repo);
            println!("Rest: {:?}", rest);
        }
        None => {
            println!("No command given");
            not_match = true
        }
    }

    // 如果已经匹配到了命令，就不需要进行后面的 fallback 逻辑了
    if !not_match {
        exit(0)
    }

    // fallback
    // 1. 查询用户输入的是否为 github.com 等或者 为某个 alias

    // 2. 查询用户输入的是否为某个用户
}
