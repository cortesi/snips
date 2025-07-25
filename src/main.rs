use clap::{Parser, Subcommand};
use log::info;
use snips::{Processor, process_file};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Enable verbose logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Quiet mode
    #[arg(long, action = clap::ArgAction::SetTrue)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Process files to sync snippets
    Render { files: Vec<PathBuf> },
    /// Check if files are in sync
    Check { files: Vec<PathBuf> },
    /// Show diff of changes
    Diff { files: Vec<PathBuf> },
}

fn print_diff(old: &str, new: &str) {
    let chunks = dissimilar::diff(old, new);
    for chunk in chunks {
        match chunk {
            dissimilar::Chunk::Equal(text) => {
                for line in text.lines() {
                    println!(" {line}");
                }
            }
            dissimilar::Chunk::Delete(text) => {
                for line in text.lines() {
                    println!("-{line}");
                }
            }
            dissimilar::Chunk::Insert(text) => {
                for line in text.lines() {
                    println!("+{line}");
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let level = if cli.quiet {
        "error"
    } else if cli.verbose > 0 {
        "debug"
    } else {
        "info"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();

    let files = match &cli.command {
        Commands::Render { files } | Commands::Check { files } | Commands::Diff { files } => {
            if files.is_empty() {
                vec![PathBuf::from("README.md")]
            } else {
                files.clone()
            }
        }
    };

    match cli.command {
        Commands::Render { .. } => {
            for path in &files {
                match process_file(path, true)? {
                    Some(_) => info!("updated {}", path.display()),
                    None => info!("{} up-to-date", path.display()),
                }
            }
        }
        Commands::Check { .. } => {
            let clean = Processor::check(&files)?;
            if !clean {
                std::process::exit(1);
            }
        }
        Commands::Diff { .. } => {
            for path in &files {
                if let Some(new) = process_file(path, false)? {
                    let old = std::fs::read_to_string(path)?;
                    print_diff(&old, &new);
                }
            }
        }
    }
    Ok(())
}
