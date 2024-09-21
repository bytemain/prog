use clap::Subcommand;
use clap_complete::Shell;

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
}
