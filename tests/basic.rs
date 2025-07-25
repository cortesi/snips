use snips::{process_file, SnipsError};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[test]
fn missing_markdown_file() {
    let path = Path::new("nope.md");
    match process_file(path, false) {
        Err(SnipsError::FileNotFound(p)) => assert_eq!(p, path),
        _ => panic!("expected file not found"),
    }
}

#[test]
fn named_snippet_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let code_path = dir.path().join("code.rs");
    fs::write(&code_path, "fn main() {}\n").unwrap();
    let md_path = dir.path().join("doc.md");
    let mut f = File::create(&md_path).unwrap();
    writeln!(f, "<!-- snips: code.rs#foo -->").unwrap();
    writeln!(f, "```rust").unwrap();
    writeln!(f, "old").unwrap();
    writeln!(f, "```").unwrap();
    drop(f);
    match process_file(&md_path, false) {
        Err(SnipsError::SnippetNotFound(p, name)) => {
            assert_eq!(p, code_path);
            assert_eq!(name, "foo");
        }
        other => panic!("unexpected {:?}", other),
    }
}

#[test]
fn dedent_snippet() {
    let dir = tempfile::tempdir().unwrap();
    let code_path = dir.path().join("code.rs");
    fs::write(&code_path, "// snips-start: foo\n    fn a() {\n        println!(\"hi\");\n    }\n// snips-end: foo\n").unwrap();
    let md_path = dir.path().join("doc.md");
    let mut f = File::create(&md_path).unwrap();
    writeln!(f, "<!-- snips: code.rs#foo -->").unwrap();
    writeln!(f, "```").unwrap();
    writeln!(f, "old").unwrap();
    writeln!(f, "```").unwrap();
    drop(f);
    process_file(&md_path, true).unwrap();
    let new_content = fs::read_to_string(&md_path).unwrap();
    assert!(new_content.contains("fn a() {\n    println!(\"hi\");\n}"));
}

#[test]
fn missing_code_fence() {
    let dir = tempfile::tempdir().unwrap();
    let code_path = dir.path().join("code.rs");
    fs::write(&code_path, "// snips-start: foo\nfn a() {}\n// snips-end: foo\n").unwrap();
    let md_path = dir.path().join("doc.md");
    let mut f = File::create(&md_path).unwrap();
    writeln!(f, "<!-- snips: code.rs#foo -->").unwrap();
    writeln!(f, "not a fence").unwrap();
    drop(f);
    match process_file(&md_path, false) {
        Err(SnipsError::MissingCodeFence(_)) => (),
        other => panic!("unexpected {:?}", other),
    }
}
