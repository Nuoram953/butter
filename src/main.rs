use clap::{Parser, Subcommand};

mod commands;

#[derive(Subcommand)]
enum Commands {
    Test {},
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
        Commands::Test {} => commands::test::handle(),
    }?;

    Ok(())
}

//create files in .config/butter
//create commands
//parse rules.yml
