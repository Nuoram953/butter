use clap::{Parser, Subcommand};

mod commands;
mod config;
mod git;
mod rules;

#[derive(Subcommand)]
enum Commands {
    Check {},
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Check {} => commands::check::handle(),
    }?;

    Ok(())
}
