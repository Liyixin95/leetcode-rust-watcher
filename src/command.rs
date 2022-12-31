use clap::Parser;

#[derive(Parser)]
pub enum Command {
    Init,
    Watch,
}
