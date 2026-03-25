use clap::{Parser, Subcommand, ValueEnum};

use crate::util::config::GameSwitcher;

// For clap to parse enum values from CLI
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum GameCli {
    HK,
    SK,
}

impl From<GameCli> for GameSwitcher {
    fn from(gc: GameCli) -> Self {
        match gc {
            GameCli::HK => GameSwitcher::HollowKnight,
            GameCli::SK => GameSwitcher::SilkSong,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "vespa",
    version,
    about = "A Cross-Platform Package Manager for the Hollow Knight Franchise"
)]
pub struct Arg {
    /// Specify which game to target
    #[arg(long, global = true, value_enum)]
    pub game: Option<GameCli>,

    #[arg(long, global = true)]
    pub profile: Option<String>,

    #[command(subcommand)]
    pub sub: SubArgs,
}

#[derive(Subcommand, Debug)]
pub enum SubArgs {
    Install {
        #[arg(required = true)]
        packages: Vec<String>,
    },
    Uninstall {
        #[arg(required = true)]
        packages: Vec<String>,

        #[arg(long)]
        force: bool,
    },
    List {
        /// Filter by package names (optional)
        #[arg(required = false)]
        packages: Vec<String>,

        #[arg(
            long,
            help = "Show available mods from the repository instead of installed ones"
        )]
        available: bool,

        #[arg(
            long,
            help = "Show all package versions, instead of just the latest"
        )]
        all_versions: bool,
    },
    Show {
        #[arg(required = true)]
        package: String,
    },
}
