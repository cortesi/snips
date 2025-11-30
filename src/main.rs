#![warn(missing_docs)]

//! Command-line interface for synchronizing snippets.

use clap::Parser;
use owo_colors::OwoColorize;
use snips::{
    RenderSummary, SnippetReport, SnipsError, diff_file, sync_snippets_in_file_with_summary,
};
use std::path::{Path, PathBuf};
use std::{env, error::Error, fs, process};

/// Available operating modes for the CLI.
enum Mode {
    /// Render snippets into files, writing changes when needed.
    Render {
        /// When true, files aren't written and exit non-zero if changes are needed.
        check: bool,
    },
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
    /// Check mode: don't write changes, exit with error if files are out of sync
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

/// Convert `path` to a string relative to `cwd` when possible.
fn relative_display(path: &Path, cwd: &Path) -> String {
    path.strip_prefix(cwd).unwrap_or(path).display().to_string()
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

    let mode = if cli.diff {
        Mode::Diff
    } else {
        Mode::Render { check: cli.check }
    };

    let cwd = env::current_dir()?;
    let files = resolve_files(&cli.files)?;

    match mode {
        Mode::Render { check } => {
            let mut any_updated = false;
            for path in &files {
                let summary: RenderSummary =
                    sync_snippets_in_file_with_summary(path, !check)?;
                let file_updated = summary.snippets.iter().any(|s| s.updated);
                any_updated = any_updated || file_updated;

                if cli.quiet {
                    continue;
                }

                let display_path = relative_display(path, &cwd);
                let file_label = format!("{}", display_path.blue().bold());
                println!("{file_label}");

                if summary.snippets.is_empty() {
                    let none = format!("{}", "(no snippets found)".bright_yellow());
                    println!("  {none}");
                } else {
                    for SnippetReport { locator, updated } in summary.snippets {
                        let marker = locator.marker();
                        let bullet = format!("{}", "↳".cyan());
                        let marker_display = if updated {
                            if check {
                                format!("{} [out of sync]", marker.red())
                            } else {
                                format!("{} [updated]", marker.green())
                            }
                        } else {
                            format!("{}", marker.bright_white().dimmed())
                        };
                        println!("  {bullet} {marker_display}");
                    }
                }
            }
            if check && any_updated {
                process::exit(1);
            }
        }
        Mode::Diff => {
            for path in &files {
                let diffs = diff_file(path)?;
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
