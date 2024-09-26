mod commands;
mod constants;
mod context;
mod helpers;

use std::io;

use std::path::PathBuf;

use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use log::info;

#[derive(Subcommand, Debug)]
pub enum ECommands {
    Clone {
        url: String,

        #[arg(allow_hyphen_values = true)]
        rest: Vec<String>,
    },
    Find {
        keyword: String,
    },
    Sync,
    Completion {
        shell: Shell,
    },
    Import {
        path: PathBuf,
    },
    Remove {
        path: PathBuf,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, allow_external_subcommands = true)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<ECommands>,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .format_timestamp(None)
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let mut config_file_path = constants::CONFIG_TOML_FILE.clone();
    if let Some(config_path) = cli.config.as_deref() {
        info!("Use specific config: {}", config_path.display());
        config_file_path = config_path.to_path_buf();
    }

    let mut context = context::Context::new(config_file_path);

    match cli.command {
        Some(ECommands::Clone { url, rest }) => commands::clone::run(&mut context, &url, &rest),
        Some(ECommands::Find { keyword }) => commands::find::run(&context, &keyword),
        Some(ECommands::Sync) => commands::sync::run(&context),
        Some(ECommands::Import { path }) => commands::import::run(&mut context, path),
        Some(ECommands::Remove { path }) => commands::remove::run(&mut context, path),
        Some(ECommands::Completion { shell }) => {
            eprintln!("Generating completion file for {shell:?}...");
            let mut cmd = Cli::command();
            let bin_name = &cmd.get_name().to_string();
            generate(shell, &mut cmd, bin_name, &mut io::stdout());
        }
        None => {}
    }
}
