mod cli;
mod commands;
mod constants;
mod context;
mod helpers;
mod internal;
mod macros;

use crate::cli::{Cli, ECommands};

fn main() {
    // use PROG_LOG="debug" to enable debug logs
    env_logger::Builder::new().parse_env("PROG_LOG").format_timestamp(None).init();

    let mut context = context::Context::new();

    let cli = Cli::new();
    match cli.command {
        Some(ECommands::Add { url, rest }) => commands::add::run(&mut context, &url, &rest),
        Some(ECommands::Find { keyword, query }) => commands::find::run(&context, &keyword, query),
        Some(ECommands::Sync) => commands::sync::run(&context),
        Some(ECommands::Import { path }) => commands::import::run(&mut context, path),
        Some(ECommands::Remove { path, yes }) => commands::remove::run(&mut context, path, yes),
        Some(ECommands::Clean { yes }) => commands::clean::run(&context, yes),
        Some(ECommands::List) => commands::list::run(&mut context),
        Some(ECommands::Size) => commands::size::run(&mut context),
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
