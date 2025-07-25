use clap::{Parser, Subcommand};
use snips::{Processor, get_snippet_diffs, process_file, SnipsError};
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
    /// Check if files are in sync (exits with error if out of date)
    Check { files: Vec<PathBuf> },
    /// Show diff of changes
    Diff { files: Vec<PathBuf> },
}

fn print_diff(old: &str, new: &str) {
    let diff = similar::TextDiff::from_lines(old, new);
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "-",
            similar::ChangeTag::Insert => "+",
            similar::ChangeTag::Equal => " ",
        };
        print!("{sign}{change}");
    }
}

fn main() {
    if let Err(e) = run() {
        if let Some(snips_error) = e.downcast_ref::<SnipsError>() {
            eprintln!("Error: {}", snips_error);
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

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
                    Some(_) => {
                        if !cli.quiet {
                            println!("updated {}", path.display());
                        }
                    }
                    None => {
                        if !cli.quiet {
                            println!("{} up-to-date", path.display());
                        }
                    }
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
                let diffs = get_snippet_diffs(path)?;
                if !diffs.is_empty() {
                    for diff in diffs {
                        let name_display = if let Some(name) = &diff.name {
                            format!("{}#{}", diff.path, name)
                        } else {
                            diff.path.clone()
                        };
                        println!("--- {name_display}");
                        println!("+++ {name_display}");
                        print_diff(&diff.old_content, &diff.new_content);
                        println!();
                    }
                }
            }
        }
    }
    Ok(())
}
