use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors produced while processing snippets.
pub enum SnipsError {
    /// Referenced source file could not be read from disk.
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
    /// A marker was not followed by a fenced code block.
    #[error("marker not followed by code fence: line {0}")]
    MissingCodeFence(usize),
    /// A marker does not match the expected syntax.
    #[error(
        "invalid marker format in {file}:{line}\n  {content}\n  Expected format: <!-- snips: path/to/file.ext --> or <!-- snips: path/to/file.ext#snippet_name -->"
    )]
    InvalidMarker {
        /// Markdown file containing the invalid marker.
        file: PathBuf,
        /// One-based line number of the marker.
        line: usize,
        /// Full text of the offending line.
        content: String,
    },
    /// A requested snippet name is missing from the source file.
    #[error(
        "snippet `{snippet_name}` not found in {file}\nAvailable snippets: {available_snippets}"
    )]
    SnippetNotFound {
        /// Source file that was scanned for the snippet.
        file: PathBuf,
        /// Name of the missing snippet.
        snippet_name: String,
        /// Comma-separated list of snippets that were found.
        available_snippets: String,
    },
    /// A snippet start marker was found without a matching end marker.
    #[error("unterminated snippet `{1}` in {0}")]
    UnterminatedSnippet(PathBuf, String),
    /// No markdown files were found in the working directory.
    #[error("no markdown files found in {0}")]
    NoMarkdownFiles(PathBuf),
    /// Any other I/O error propagated from the filesystem.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
