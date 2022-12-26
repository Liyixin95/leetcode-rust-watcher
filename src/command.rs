use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub enum Command {
    Workspace {
        #[command(subcommand)]
        command: Workspace,
    },
    Daemon {
        #[command(subcommand)]
        command: Daemon,
    },
}

#[derive(Subcommand)]
pub enum Workspace {
    Init {
        #[arg(default_value = "./")]
        path: PathBuf,
    },
    Watch {
        #[arg(default_value = "./")]
        path: PathBuf,
    },
    List {
        #[arg(default_value_t = 30000)]
        port: u32,
    },
    Delete,
}

#[derive(Subcommand)]
pub enum Daemon {
    Start(StartConfig),
    Stop {
        #[arg(default_value_t = 30000)]
        port: u32,
    },
}

#[derive(Args)]
pub struct StartConfig {
    #[arg(default_value_t = 30000)]
    pub port: u16,
}
