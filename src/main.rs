#![feature(path_add_extension)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{fs::File, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Config {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Compress {
        input: PathBuf,
        #[arg(long, short)]
        output: Option<PathBuf>,
        #[arg(long, value_parser=clap::value_parser!(u8).range(2..17))]
        word_size: u8,
    },
    Decompress {
        input: PathBuf,
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
}

fn run() -> Result<()> {
    let args = Config::parse();

    match args.command {
        Commands::Compress {
            word_size,
            input,
            output,
        } => {
            let output = File::create(output.unwrap_or(input.clone().with_added_extension("hf")))?;
            let input = File::open(input)?;

            zip::compress(word_size, input, output)?;
        }
        Commands::Decompress { input, output } => {
            let output = File::create(output.unwrap_or(input.clone().with_extension("")))?;
            let input = File::open(input)?;

            zip::decompress(input, output)?;
        }
    }

    Ok(())
}

fn main() {
    run().unwrap();
}