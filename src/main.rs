use crate::operator::Operator;
use crate::watcher::watch;
use std::env;

mod command;
mod mapping;
mod operator;
mod watcher;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cur_path = env::current_dir()?;
    let (_watcher, rx) = watch(&cur_path)?;

    let operator = Operator::new(cur_path, rx)?;

    operator.run()
}
