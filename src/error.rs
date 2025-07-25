use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SnipsError {
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
    #[error("marker not followed by code fence: line {0}")]
    MissingCodeFence(usize),
    #[error("invalid marker format in {file}:{line}\n  {content}\n  Expected format: <!-- snips: path/to/file.ext --> or <!-- snips: path/to/file.ext#snippet_name -->")]
    InvalidMarker {
        file: PathBuf,
        line: usize,
        content: String,
    },
    #[error("snippet `{snippet_name}` not found in {file}\nAvailable snippets: {available_snippets}")]
    SnippetNotFound {
        file: PathBuf,
        snippet_name: String,
        available_snippets: String,
    },
    #[error("unterminated snippet `{1}` in {0}")]
    UnterminatedSnippet(PathBuf, String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
