pub mod error;
pub mod processor;
pub mod snippet;

pub use error::SnipsError;
pub use processor::{Processor, SnippetDiff, get_snippet_diffs, process_file};
