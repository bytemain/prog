use std::path::PathBuf;

use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io::{self, Write};

use crate::{commands, helpers::template::render_template};

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
        #[arg(short = 'q', long = "query", help = "Only query result")]
        query: bool,
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
    Tmp(commands::tmp::TmpArgs),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, allow_external_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<ECommands>,
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

impl Cli {
    pub fn activate(shell: Shell) {
        let mut cmd = Cli::command();
        let bin_name = &cmd.get_name().to_string();

        // get all commands and make a vec
        let mut commands = Vec::new();
        for subcommand in cmd.get_subcommands() {
            commands.push(subcommand.get_name());
        }
        // add builtin help
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

        let context = crate::collection! {
            String::from("if_check_statement") => if_check_statement,
            String::from("command") => String::from(command),
        };

        let text = render_template(String::from(script_content), &context);

        generate(shell, &mut cmd, bin_name, &mut io::stdout());
        generate(shell, &mut cmd, command, &mut io::stdout());
        io::stdout().write_all(text.as_bytes()).expect("Could not write to stdout");
    }

    pub fn show_help() {
        let mut cmd = Cli::command();
        cmd.print_help().expect("Could not print help");
        std::process::exit(1);
    }

    pub fn get_subcommand(sub_cmd: &str) -> Option<Command> {
        let mut cmd = Cli::command();

        for subcommand in cmd.get_subcommands_mut() {
            // Use get_subcommands_mut() if you need to modify, or clone directly if Command is Clone
            if subcommand.get_name() == sub_cmd {
                return Some(subcommand.clone()); // Clone the subcommand
            }
        }
        None
    }
}
