#![warn(missing_docs)]

//! Command-line interface for synchronizing snippets.

use clap::Parser;
use snips::{Processor, SnipsError, get_snippet_diffs, process_file};
use std::path::PathBuf;
use std::{error::Error, process};

/// Available operating modes for the CLI.
enum Mode {
    /// Render snippets into files, writing changes when needed.
    Render,
    /// Check whether files are up to date without writing.
    Check,
    /// Display diffs between embedded snippets and sources.
    Diff,
}

#[derive(Parser)]
#[command(version, about)]
/// Parsed command-line arguments.
struct Cli {
    /// Quiet mode
    #[arg(long, action = clap::ArgAction::SetTrue)]
    quiet: bool,
    /// Check if files are in sync (exits with error if out of date)
    #[arg(long, action = clap::ArgAction::SetTrue, conflicts_with = "diff")]
    check: bool,
    /// Show diff of changes
    #[arg(long, action = clap::ArgAction::SetTrue, conflicts_with = "check")]
    diff: bool,
    /// Files to process
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

/// Show a unified diff between two string slices.
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

/// Program entry point.
fn main() {
    if let Err(e) = run() {
        if let Some(snips_error) = e.downcast_ref::<SnipsError>() {
            eprintln!("Error: {snips_error}");
        } else {
            eprintln!("Error: {e}");
        }
        process::exit(1);
    }
}

/// Execute the command selected by CLI arguments.
fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mode = if cli.check {
        Mode::Check
    } else if cli.diff {
        Mode::Diff
    } else {
        Mode::Render
    };

    match mode {
        Mode::Render => {
            for path in &cli.files {
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
        Mode::Check => {
            let clean = Processor::check(&cli.files)?;
            if !clean {
                process::exit(1);
            }
        }
        Mode::Diff => {
            for path in &cli.files {
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
