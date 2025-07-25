pub mod processor;
pub mod snippet;
pub mod error;

pub use processor::{process_file, Processor};
pub use error::SnipsError;
