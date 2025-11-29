#![warn(missing_docs)]

//! Command-line interface for synchronizing snippets.

use clap::Parser;
use snips::{Processor, SnipsError, get_snippet_diffs, process_file};
use std::path::{Path, PathBuf};
use std::{env, error::Error, fs, process};

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
    /// Files to process; defaults to all markdown files in the current directory when omitted.
    #[arg(num_args = 0..)]
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

/// Determine whether `path` points to a markdown file.
fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "md" | "markdown"))
        .unwrap_or(false)
}

/// Determine which files to operate on, defaulting to all markdown files in the CWD.
fn resolve_files(cli_files: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    if !cli_files.is_empty() {
        return Ok(cli_files.to_vec());
    }

    let cwd = env::current_dir()?;
    let mut discovered = Vec::new();
    for entry in fs::read_dir(&cwd)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && is_markdown(&path) {
            discovered.push(path);
        }
    }

    discovered.sort();
    if discovered.is_empty() {
        return Err(Box::new(SnipsError::NoMarkdownFiles(cwd)));
    }

    Ok(discovered)
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

    let files = resolve_files(&cli.files)?;

    match mode {
        Mode::Render => {
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
        Mode::Check => {
            let clean = Processor::check(&files)?;
            if !clean {
                process::exit(1);
            }
        }
        Mode::Diff => {
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
