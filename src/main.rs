use std::process;

use clap::{Parser, Subcommand};

mod commands;
mod config;
mod git;
mod output;
mod rules;

#[derive(Subcommand)]
enum Commands {
    Check {
        #[arg(short, long)]
        branch: Option<String>,
    },
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

    if !git::is_git_repo() {
        println!("not a git repo");
        process::exit(1)
    }

    let cli = Cli::parse();

    match &cli.command {
        Commands::Check { branch } => commands::check::handle(branch.as_deref()),
    }?;

    Ok(())
}
