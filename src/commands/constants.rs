use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum ECommands {
    Clone {
        url: String,

        #[arg(allow_hyphen_values = true)]
        rest: Vec<String>,
    },
    Query {
        keyword: String
    }
}
