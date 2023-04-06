mod commands;
mod configuration;
mod context;
mod helpers;

use config::Config;

use ansi_term::Colour::Red;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<commands::constants::ECommands>,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
    let cli = Cli::parse();

    let mut config_file_path_string =
        helpers::path::join_home_dir(configuration::DEFAULT_CONFIG_TOML_PATH);

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

    let config = config_builder
        .try_deserialize::<configuration::Config>()
        .unwrap();
    println!("{:?}", config);

    if config.base.len() == 0 {
        println!(
            "No base path found, please add one to your config file: {}",
            config_file_path_string
        );
        exit(1)
    }

    let context = context::Context::new(&config);

    let mut not_match = false;
    match &cli.command {
        Some(commands::constants::ECommands::Clone { url: repo, rest }) => {
            println!("Clone command given");
            commands::clone::run(&context, &repo, &rest)
        }
        None => {
            println!("{}", Red.paint("No command given"));
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
