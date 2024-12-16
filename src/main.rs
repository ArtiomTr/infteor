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
        #[arg(long)]
        prune: i64,
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
            prune,
            input,
            output,
        } => {
            let output = File::create(output.unwrap_or(input.clone().with_added_extension("hf")))?;
            let input = File::open(input)?;

            // zip::compress(word_size, input, output)?;
            lz78::encode(input, output, prune)?;
        }
        Commands::Decompress { input, output } => {
            let output = File::create(output.unwrap_or(input.clone().with_extension("")))?;
            let input = File::open(input)?;

            lz78::decode(input, output)?;
        }
    }

    Ok(())
}

fn main() {
    run().unwrap();
}
