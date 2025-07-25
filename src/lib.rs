pub mod error;
pub mod processor;
pub mod snippet;

pub use error::SnipsError;
pub use processor::{Processor, process_file};
