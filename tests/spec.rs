//! Specification-style integration tests.

/// Validate expected behavior across the documented spec cases.
#[cfg(test)]
mod tests {
    use snips::{SnipsError, process_file};
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    // Helper to write a markdown with marker and code fence
    fn write_marker(path: &Path, marker: &str) {
        let mut f = File::create(path).unwrap();
        writeln!(f, "{marker}").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
    }

    // Helper to write a markdown with trailing spaces on the closing fence
    fn write_marker_trailing(path: &Path, marker: &str, spaces: &str) {
        let mut f = File::create(path).unwrap();
        writeln!(f, "{marker}").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```{spaces}").unwrap();
    }

    #[test]
    fn relative_path_resolution() {
        let dir = tempfile::tempdir().unwrap();
        let src_dir = dir.path().join("src");
        let docs_dir = dir.path().join("docs");
        fs::create_dir(&src_dir).unwrap();
        fs::create_dir(&docs_dir).unwrap();
        let code_path = src_dir.join("code.rs");
        fs::write(&code_path, "fn x() {}\n").unwrap();
        let md_path = docs_dir.join("doc.md");
        write_marker(&md_path, "<!-- snips: ../src/code.rs -->");
        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();
        assert!(content.contains("fn x() {}"));
    }

    #[test]
    fn malformed_marker() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: -->").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);
        match process_file(&md_path, false) {
            Err(SnipsError::InvalidMarker { .. }) => (),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn no_markers() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("doc.md");
        fs::write(&md_path, "no snippets\n").unwrap();
        let res = process_file(&md_path, false).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn unterminated_snippet() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        fs::write(&code_path, "// snips-start: foo\nfn a(){}\n").unwrap();
        let md_path = dir.path().join("doc.md");
        write_marker(&md_path, "<!-- snips: code.rs#foo -->");
        match process_file(&md_path, false) {
            Err(SnipsError::UnterminatedSnippet(p, name)) => {
                assert_eq!(p, code_path);
                assert_eq!(name, "foo");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn mismatched_snippet_names() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        fs::write(&code_path, "// snips-start: A\nfn a(){}\n// snips-end: B\n").unwrap();
        let md_path = dir.path().join("doc.md");
        write_marker(&md_path, "<!-- snips: code.rs#A -->");
        match process_file(&md_path, false) {
            Err(SnipsError::UnterminatedSnippet(p, name)) => {
                assert_eq!(p, code_path);
                assert_eq!(name, "A".to_string());
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn empty_named_snippet() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        fs::write(&code_path, "// snips-start: foo\n// snips-end: foo\n").unwrap();
        let md_path = dir.path().join("doc.md");
        write_marker(&md_path, "<!-- snips: code.rs#foo -->");
        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();
        assert!(content.contains("```"));
    }

    #[test]
    fn no_indentation() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        fs::write(
            &code_path,
            "// snips-start: foo\nfn a(){}\n// snips-end: foo\n",
        )
        .unwrap();
        let md_path = dir.path().join("doc.md");
        write_marker(&md_path, "<!-- snips: code.rs#foo -->");
        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();
        assert!(content.contains("fn a(){}"));
    }

    #[test]
    fn inconsistent_indentation() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        let code = "// snips-start: foo\n    fn a(){}\n fn b(){}\n    // snips-end: foo\n";
        fs::write(&code_path, code).unwrap();
        let md_path = dir.path().join("doc.md");
        write_marker(&md_path, "<!-- snips: code.rs#foo -->");
        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();
        assert!(content.contains("fn b(){}"));
        assert!(content.contains("fn a(){}"));
    }

    #[test]
    fn whitespace_lines_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        let code = "// snips-start: foo\n    fn a(){}\n\n    fn b(){}\n// snips-end: foo\n";
        fs::write(&code_path, code).unwrap();
        let md_path = dir.path().join("doc.md");
        write_marker(&md_path, "<!-- snips: code.rs#foo -->");
        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();
        assert!(content.contains("fn a(){}\n\nfn b(){}"));
    }

    #[test]
    fn whole_file_insertion() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        fs::write(&code_path, "fn main(){}\n").unwrap();
        let md_path = dir.path().join("doc.md");
        write_marker(&md_path, "<!-- snips: code.rs -->");
        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();
        assert!(content.contains("fn main(){}"));
    }

    #[test]
    fn whole_file_empty() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        fs::write(&code_path, "").unwrap();
        let md_path = dir.path().join("doc.md");
        write_marker(&md_path, "<!-- snips: code.rs -->");
        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();
        assert!(content.contains("```"));
    }

    #[test]
    fn unknown_extension() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.data");
        fs::write(&code_path, "foo\n").unwrap();
        let md_path = dir.path().join("doc.md");
        write_marker(&md_path, "<!-- snips: code.data -->");
        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();
        assert!(content.contains("```\nfoo"));
    }

    #[test]
    fn idempotent_processing() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        fs::write(&code_path, "fn main(){}\n").unwrap();
        let md_path = dir.path().join("doc.md");
        write_marker(&md_path, "<!-- snips: code.rs -->");
        // first run
        process_file(&md_path, true).unwrap();
        let first = fs::read_to_string(&md_path).unwrap();
        // second run
        let res = process_file(&md_path, true).unwrap();
        assert!(res.is_none());
        let second = fs::read_to_string(&md_path).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn closing_fence_trailing_spaces() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        fs::write(&code_path, "fn main(){}\n").unwrap();
        let md_path = dir.path().join("doc.md");
        write_marker_trailing(&md_path, "<!-- snips: code.rs -->", "   ");
        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();
        assert!(content.contains("fn main(){}"));
    }

    #[test]
    fn multiple_markers_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let code1 = dir.path().join("a.rs");
        let code2 = dir.path().join("b.rs");
        fs::write(&code1, "fn a(){}\n").unwrap();
        fs::write(&code2, "fn b(){}\n").unwrap();
        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: a.rs -->").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "<!-- snips: b.rs -->").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        process_file(&md_path, true).unwrap();
        let first = fs::read_to_string(&md_path).unwrap();
        assert!(first.contains("fn a(){}"));
        assert!(first.contains("fn b(){}"));

        let res = process_file(&md_path, true).unwrap();
        assert!(res.is_none());
        let second = fs::read_to_string(&md_path).unwrap();
        assert_eq!(first, second);
    }
}
