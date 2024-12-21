mod commands;
mod constants;
mod context;
mod helpers;
mod schema;

use std::io::{self, Write};

use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};

#[derive(Subcommand, Debug)]
pub enum ECommands {
    #[command(about = "Add a new repository")]
    Add {
        url: String,
        #[arg(allow_hyphen_values = true)]
        rest: Vec<String>,
    },
    #[command(about = "Find a repository by keyword")]
    Find {
        keyword: String,
    },
    Query {
        keyword: String,
    },
    #[command(about = "Sync repositories")]
    Sync,
    #[command(about = "Activate shell")]
    Shell {
        shell: Shell,
    },
    #[command(about = "Import repositories from a path")]
    Import {
        path: PathBuf,
    },
    #[command(about = "Remove a repository by path")]
    Remove {
        path: PathBuf,
    },
    #[command(about = "Clean up repositories")]
    Clean,
    #[command(about = "List all repositories")]
    List,
    #[command(about = "Initialize configuration")]
    Init,
    #[command(about = "Create a temporary directory")]
    Cdtmp,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, allow_external_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<ECommands>,
}

impl Cli {
    fn activate(shell: Shell) {
        let mut cmd = Cli::command();
        let bin_name = &cmd.get_name().to_string();
        generate(shell, &mut cmd, bin_name, &mut io::stdout());
        let bytes = include_bytes!("shell-integrations/zsh");
        io::stdout().write_all(bytes).expect("Could not write to stdout");
    }
}

fn show_help() {
    let mut cmd = Cli::command();
    cmd.print_help().expect("Could not print help");
    std::process::exit(1);
}

fn main() {
    let cli = Cli::parse();
    // use PROG_LOG="debug" to enable debug logs
    env_logger::Builder::new().parse_env("PROG_LOG").format_timestamp(None).init();

    let mut context = context::Context::new();

    match cli.command {
        Some(ECommands::Add { url, rest }) => commands::add::run(&mut context, &url, &rest),
        Some(ECommands::Find { keyword }) => {
            commands::find::check_keyword_exists(&context, &keyword);
        }
        Some(ECommands::Query { keyword }) => {
            commands::find::query(&context, &keyword);
        }
        Some(ECommands::Sync) => commands::sync::run(&context),
        Some(ECommands::Import { path }) => commands::import::run(&mut context, path),
        Some(ECommands::Remove { path }) => commands::remove::run(&mut context, path),
        Some(ECommands::Clean) => commands::clean::run(&mut context),
        Some(ECommands::List) => commands::list::run(&mut context),
        Some(ECommands::Init) => commands::init::run(&mut context),
        Some(ECommands::Cdtmp) => commands::cdtmp::run(&mut context),
        Some(ECommands::Shell { shell }) => Cli::activate(shell),
        None => show_help(),
    }
    ()
}
