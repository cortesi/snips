use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Write a marker block with a single code fence and body.
pub fn write_marker_block(
    path: &Path,
    marker: &str,
    fence: &str,
    body: &str,
    closing_suffix: &str,
) {
    let mut f = File::create(path).unwrap();
    writeln!(f, "{marker}").unwrap();
    writeln!(f, "{fence}").unwrap();
    if !body.is_empty() {
        writeln!(f, "{body}").unwrap();
    }
    writeln!(f, "{fence}{closing_suffix}").unwrap();
}

/// Write a marker block using triple backticks and default body "old".
pub fn write_marker(path: &Path, marker: &str) {
    write_marker_block(path, marker, "```", "old", "");
}

/// Write a marker block with trailing characters on the closing fence.
pub fn write_marker_with_suffix(path: &Path, marker: &str, suffix: &str) {
    write_marker_block(path, marker, "```", "old", suffix);
}

/// Create a source file containing a named snippet section.
pub fn write_source_with_snippet(path: &Path, name: &str, content: &str) {
    let mut f = File::create(path).unwrap();
    writeln!(f, "// Some code before").unwrap();
    writeln!(f, "// snips-start: {name}").unwrap();
    write!(f, "{content}").unwrap();
    writeln!(f, "// snips-end: {name}").unwrap();
    writeln!(f, "// Some code after").unwrap();
}

/// Write an entire source file.
pub fn write_source_file(path: &Path, content: &str) {
    let mut f = File::create(path).unwrap();
    write!(f, "{content}").unwrap();
}

/// Create a minimal markdown+source pair used by CLI tests.
pub fn make_example(dir: &TempDir) -> PathBuf {
    let code = dir.path().join("code.rs");
    write_source_file(&code, "fn main(){}\n");

    let md = dir.path().join("README.md");
    write_marker(&md, "<!-- snips: code.rs -->");
    md
}
