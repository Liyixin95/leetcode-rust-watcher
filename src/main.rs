use crate::command::Command;
use crate::operator::Operator;
use crate::watcher::watch;
use crate::workspace::init_workspace;
use clap::Parser;
use std::env;
use std::path::PathBuf;

mod command;
mod mapping;
mod operator;
mod watcher;
mod workspace;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cur_path = env::current_dir()?;

    let command = Command::parse();
    match command {
        Command::Init => init_workspace(cur_path),
        Command::Watch => {
            init_workspace(cur_path.clone())?;
            start_watch(cur_path)
        }
    }
}

fn start_watch(cur_path: PathBuf) -> anyhow::Result<()> {
    let (_watcher, rx) = watch(&cur_path)?;

    let operator = Operator::new(cur_path, rx)?;

    operator.run()
}
