use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SnipsError {
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
    #[error("marker not followed by code fence: line {0}")]
    MissingCodeFence(usize),
    #[error("invalid marker format at line {0}")]
    InvalidMarker(usize),
    #[error("snippet `{1}` not found in {0}")]
    SnippetNotFound(PathBuf, String),
    #[error("unterminated snippet `{1}` in {0}")]
    UnterminatedSnippet(PathBuf, String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
