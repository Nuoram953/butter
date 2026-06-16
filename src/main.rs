use std::fs::File;
use std::path::PathBuf;
use std::{fs, io};

use clap::{Parser, Subcommand};
use directories::ProjectDirs;

mod commands;
mod config;

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
    let cli = Cli::parse();

    match &cli.command {
        Commands::Check {} => commands::test::handle(),
    }?;

    Ok(())
}

//create files in .config/butter
//create commands
//parse rules.yml
