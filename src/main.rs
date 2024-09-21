mod commands;
mod configuration;
mod constants;
mod context;
mod helpers;

use config::Config;
use std::io;

use ansi_term::Colour::Red;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

use commands::constants::ECommands;
use clap::{Args, Command, CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{generate, Generator, Shell};
use log::{debug, error, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<ECommands>,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn main() {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .format_timestamp(None)
        .filter_level(cli.verbose.log_level_filter())
        .init();


    let mut config_file_path = helpers::path::get_config_path(constants::CONFIG_TOML_FILE);
    if let Some(config_path) = cli.config.as_deref() {
        info!("Use specific config: {}", config_path.display());
        config_file_path = config_path.to_path_buf();
    }

    let mut context = context::Context::new(config_file_path);

    match cli.command {
        Some(ECommands::Clone { url, rest }) => commands::clone::run(&mut context, &url, &rest),
        Some(ECommands::Find { keyword }) => commands::find::run(&context, &keyword),
        Some(ECommands::Sync) => commands::sync::run(&context),
        Some(ECommands::Completion { shell }) => {
            let mut cmd = Cli::command();
            error!("Generating completion file for {shell:?}...");
            print_completions(shell, &mut cmd);
            return;
        }
        None => {
            // fallback
            // 1. 查询用户输入的是否为 github.com 等或者 为某个 alias
            // 2. 查询用户输入的是否为某个用户
            let mut cmd = Cli::command();
            cmd.print_help().expect("Failed to print help");
        }
    }
}
