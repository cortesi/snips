use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
/// Errors produced while processing snippets.
pub enum SnipsError {
    /// Referenced source file could not be read from disk.
    #[error("file not found: {file}")]
    FileNotFound {
        /// File that could not be read.
        file: PathBuf,
        /// Underlying OS error for additional context.
        #[source]
        source: io::Error,
    },
    /// Reading a file failed for reasons other than missing file.
    #[error("failed to read {file}: {source}")]
    FileReadFailed {
        /// File that could not be read.
        file: PathBuf,
        /// Underlying OS error.
        #[source]
        source: io::Error,
    },
    /// A marker was not followed by a fenced code block.
    #[error("marker not followed by code fence: line {0}")]
    MissingCodeFence(usize),
    /// A code fence was opened but never closed.
    #[error("code fence starting at line {start_line} in {file} is missing its closing fence")]
    UnterminatedCodeFence {
        /// Markdown file containing the unterminated fence.
        file: PathBuf,
        /// One-based line number where the fence opens.
        start_line: usize,
    },
    /// A marker does not match the expected syntax.
    #[error(
        "invalid marker format in {file}:{line}\n  {content}\n  Expected format: <!-- snips: path/to/file.ext -->, <!-- snips: path/to/file.ext#snippet_name -->, or <!-- snips: !command -->"
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
    /// Command execution was disabled by policy.
    #[error("command execution disabled: {command}")]
    CommandExecutionDisabled {
        /// Command that was blocked.
        command: String,
    },
    /// Command execution was denied by the user.
    #[error("command execution denied: {command}")]
    CommandExecutionDenied {
        /// Command that was denied.
        command: String,
    },
    /// Prompting for command execution was not possible.
    #[error("command execution requires confirmation but stdin is not interactive: {command}")]
    CommandConfirmationUnavailable {
        /// Command awaiting confirmation.
        command: String,
    },
    /// Spawning a command failed.
    #[error("failed to execute command `{command}`: {source}")]
    CommandSpawnFailed {
        /// Command that failed to start.
        command: String,
        /// Underlying IO error.
        #[source]
        source: io::Error,
    },
    /// Command returned a non-zero exit status.
    #[error("command `{command}` failed with status {status}: {stderr}")]
    CommandFailed {
        /// Command that returned non-zero.
        command: String,
        /// Exit status description.
        status: String,
        /// Captured stderr output (may be empty).
        stderr: String,
    },
    /// No markdown files were found in the working directory.
    #[error("no markdown files found in {0}")]
    NoMarkdownFiles(PathBuf),
    /// Any other I/O error propagated from the filesystem.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
