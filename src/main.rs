mod cli;
mod commands;
mod constants;
mod context;
mod helpers;
mod internal;
mod macros;

use clap::Parser;

use crate::cli::{Cli, ECommands};

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
                let cmd = Cli::get_subcommand("tmp");
                if let Some(mut cmd) = cmd {
                    cmd.print_help().expect("Could not print help");
                    return;
                }

                std::process::exit(1);
            }
            commands::tmp::run(&mut context, &tmp_cmd.unwrap());
        }
        Some(ECommands::Shell { shell }) => Cli::activate(shell),
        None => Cli::show_help(),
    }
}
