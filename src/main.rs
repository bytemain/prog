mod commands;
mod constants;
mod context;
mod helpers;
mod schema;

use std::env::args;
use std::io;

use std::path::PathBuf;

use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use log::info;

#[derive(Subcommand, Debug)]
pub enum ECommands {
    #[command(about = "Add a new repository")]
    Add {
        url: String,

        #[arg(allow_hyphen_values = true)]
        rest: Vec<String>,
    },
    #[command(about = "Find a repository by keyword")]
    Find { keyword: String },
    #[command(about = "Sync repositories")]
    Sync,
    #[command(about = "Generate shell completion scripts")]
    Completion { shell: Shell },
    #[command(about = "Import repositories from a path")]
    Import { path: PathBuf },
    #[command(about = "Remove a repository by path")]
    Remove { path: PathBuf },
    #[command(about = "Clean up repositories")]
    Clean,
    #[command(about = "List all repositories")]
    List,
    #[command(about = "Initialize configuration")]
    Init,
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

fn show_help() {
    eprintln!("No command provided");
    let mut cmd = Cli::command();
    cmd.print_help().expect("Could not print help");
    std::process::exit(1);
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
        Some(ECommands::Add { url, rest }) => commands::add::run(&mut context, &url, &rest),
        Some(ECommands::Find { keyword }) => commands::find::run(&context, &keyword),
        Some(ECommands::Sync) => commands::sync::run(&context),
        Some(ECommands::Import { path }) => commands::import::run(&mut context, path),
        Some(ECommands::Remove { path }) => commands::remove::run(&mut context, path),
        Some(ECommands::Clean) => commands::clean::run(&mut context),
        Some(ECommands::List) => commands::list::run(&mut context),
        Some(ECommands::Init) => commands::init::run(&mut context),
        Some(ECommands::Completion { shell }) => {
            let mut cmd = Cli::command();
            let bin_name = &cmd.get_name().to_string();
            generate(shell, &mut cmd, bin_name, &mut io::stdout());
        }
        None => {
            // find the first subcommand in database
            let args_0 = args().nth(1);
            match args_0 {
                Some(subcommand) => {
                    let matched = commands::find::run_from_not_matched(&context, &subcommand);
                    if !matched {
                        show_help();
                    }
                }
                None => show_help(),
            }
        }
    }
}
