mod commands;
mod constants;
mod context;
mod helpers;
mod internal;
#[macro_use]
mod macros;

use std::io::{self, Write};

use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use helpers::template::render_template;

#[derive(Subcommand, Debug)]
enum ECommands {
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
        #[arg(short = 'y', long = "yes", help = "Skip confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Clean up repositories")]
    Clean {
        #[arg(short = 'y', long = "yes", help = "Skip confirmation prompt")]
        yes: bool,
    },
    #[command(about = "List all repositories")]
    List,
    Tmp(TmpArgs),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct TmpArgs {
    #[command(subcommand)]
    command: Option<TmpCommands>,
}

#[derive(Debug, Subcommand)]
enum TmpCommands {
    #[command(about = "Clean temporary directories")]
    Clean,
    #[command(about = "Create a temporary directory")]
    Create,
    #[command(about = "List temporary directories")]
    List,
}

trait ShellScriptName {
    /// Returns the content of the corresponding shell integration script
    fn integration_script(&self) -> &'static str;
}

impl ShellScriptName for Shell {
    fn integration_script(&self) -> &'static str {
        match self {
            Shell::Bash => include_str!("shell-integrations/bash.sh"),
            Shell::PowerShell => include_str!("shell-integrations/powershell.ps1"),
            Shell::Zsh => include_str!("shell-integrations/zsh.sh"),
            _ => "",
        }
    }
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

        // get all commands and make a vec
        let mut commands = Vec::new();
        for subcommand in cmd.get_subcommands() {
            commands.push(subcommand.get_name());
        }
        // add bultin help
        commands.push("help");

        // transform commands to if check
        // [[ "$1" = commands.0 ]] || [[ "$1" = commands.1 ]] ||...
        let mut if_check = vec![];
        for command in commands {
            if_check.push(format!("[[ \"$1\" = \"{}\" ]]", command));
        }
        let if_check_statement = if_check.join(" || ");

        let command = "p";

        let script_content = shell.integration_script();

        if script_content.is_empty() {
            eprintln!("Shell integration for {:?} is not supported yet.", shell);
            std::process::exit(1);
        }

        let context = collection! {
            String::from("if_check_statement") => if_check_statement,
            String::from("command") => String::from(command),
        };

        let text = render_template(String::from(script_content), &context);

        generate(shell, &mut cmd, bin_name, &mut io::stdout());
        generate(shell, &mut cmd, command, &mut io::stdout());
        io::stdout().write_all(text.as_bytes()).expect("Could not write to stdout");
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

    if context.database().size() == 0 {
        context.sync_silent();
    }

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
        Some(ECommands::Remove { path, yes }) => commands::remove::run(&mut context, path, yes),
        Some(ECommands::Clean { yes }) => commands::clean::run(&context, yes),
        Some(ECommands::List) => commands::list::run(&mut context),
        Some(ECommands::Tmp(tmp)) => {
            let tmp_cmd = tmp.command;
            if tmp_cmd.is_none() {
                let mut cmd = Cli::command();
                cmd.get_subcommands_mut().for_each(|subcommand| {
                    if subcommand.get_name() == "tmp" {
                        println!("{}", subcommand);
                        subcommand.print_help().expect("Could not print help");
                    }
                });

                std::process::exit(1);
            }

            match tmp_cmd.unwrap() {
                TmpCommands::Clean => commands::tmp::cleanoutdate(&mut context),
                TmpCommands::Create => commands::tmp::run(&mut context),
                TmpCommands::List => commands::tmp::list_files(&mut context),
            }
        }
        Some(ECommands::Shell { shell }) => Cli::activate(shell),
        None => show_help(),
    }
}
