//! Integration tests for hyphenated snippet identifiers.

/// Exercises handling of hyphenated snippet names across contexts.
#[cfg(test)]
mod tests {
    use snips::{SnipsError, process_file};
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    /// Helper to build a source file containing a single hyphenated snippet.
    fn write_source_with_hyphenated_snippet(path: &Path, name: &str, content: &str) {
        let mut f = File::create(path).unwrap();
        writeln!(f, "// Some code before").unwrap();
        writeln!(f, "// snips-start: {name}").unwrap();
        write!(f, "{content}").unwrap();
        writeln!(f, "// snips-end: {name}").unwrap();
        writeln!(f, "// Some code after").unwrap();
    }

    #[test]
    fn hyphenated_snippet_names() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_with_hyphenated_snippet(
            &code_path,
            "my-example",
            "fn test_hyphen() {\n    println!(\"hyphenated!\");\n}\n",
        );

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "Example with hyphenated name:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "<!-- snips: code.rs#my-example -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old content").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify the snippet was processed correctly
        assert!(content.contains("<!-- snips: code.rs#my-example -->"));
        assert!(content.contains("fn test_hyphen() {"));
        assert!(content.contains("println!(\"hyphenated!\");"));
    }

    #[test]
    fn mixed_underscore_and_hyphen_names() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        let mut f = File::create(&code_path).unwrap();
        writeln!(f, "// snips-start: under_score").unwrap();
        writeln!(f, "fn underscore_example() {{}}").unwrap();
        writeln!(f, "// snips-end: under_score").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "// snips-start: with-hyphen").unwrap();
        writeln!(f, "fn hyphen_example() {{}}").unwrap();
        writeln!(f, "// snips-end: with-hyphen").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "// snips-start: mixed_style-name").unwrap();
        writeln!(f, "fn mixed_example() {{}}").unwrap();
        writeln!(f, "// snips-end: mixed_style-name").unwrap();
        drop(f);

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: code.rs#under_score -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "<!-- snips: code.rs#with-hyphen -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "<!-- snips: code.rs#mixed_style-name -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify all snippet styles work
        assert!(content.contains("fn underscore_example() {}"));
        assert!(content.contains("fn hyphen_example() {}"));
        assert!(content.contains("fn mixed_example() {}"));
    }

    #[test]
    fn hyphenated_names_in_indented_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_with_hyphenated_snippet(
            &code_path,
            "test-indented",
            "fn indented_test() {}\n",
        );

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "1. First item").unwrap();
        writeln!(f, "2. Item with code:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "   <!-- snips: code.rs#test-indented -->").unwrap();
        writeln!(f, "   ```rust").unwrap();
        writeln!(f, "   old").unwrap();
        writeln!(f, "   ```").unwrap();
        drop(f);

        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify indented hyphenated snippet works
        assert!(content.contains("   <!-- snips: code.rs#test-indented -->"));
        assert!(content.contains("   fn indented_test() {}"));
    }

    #[test]
    fn error_message_with_hyphenated_names() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_with_hyphenated_snippet(&code_path, "existing-snippet", "fn exists() {}\n");

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: code.rs#non-existent-snippet -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        match process_file(&md_path, false) {
            Err(SnipsError::SnippetNotFound {
                snippet_name,
                available_snippets,
                ..
            }) => {
                assert_eq!(snippet_name, "non-existent-snippet");
                assert!(available_snippets.contains("existing-snippet"));
            }
            other => panic!("Expected SnippetNotFound error, got: {:?}", other),
        }
    }
}
