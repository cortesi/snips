#![warn(missing_docs)]

//! Snips keeps markdown snippets synchronized with their source files.

/// Error definitions used across the crate.
pub mod error;
/// Core processing logic for scanning and updating markdown files.
pub mod processor;
/// Helpers for locating and extracting snippets from source files.
pub mod snippet;

pub use error::SnipsError;
pub use processor::{
    RenderSummary, SnippetDiff, SnippetLocator, SnippetReport, check_files, diff_file,
    sync_snippets_in_file, sync_snippets_in_file_with_summary,
};
