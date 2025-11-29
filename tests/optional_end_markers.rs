//! Integration tests for optional end markers.

/// Ensure colonless and optional end markers behave as expected.
#[cfg(test)]
mod tests {
    use snips::sync_snippets_in_file;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    // Helper to write a source file with optional end marker
    fn write_source_with_optional_end(
        path: &Path,
        name: &str,
        content: &str,
        include_name_in_end: bool,
    ) {
        let mut f = File::create(path).unwrap();
        writeln!(f, "// Some code before").unwrap();
        writeln!(f, "// snips-start: {name}").unwrap();
        write!(f, "{content}").unwrap();
        if include_name_in_end {
            writeln!(f, "// snips-end: {name}").unwrap();
        } else {
            writeln!(f, "// snips-end:").unwrap();
        }
        writeln!(f, "// Some code after").unwrap();
    }

    #[test]
    fn optional_end_marker_name() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_with_optional_end(
            &code_path,
            "example",
            "fn test() {\n    println!(\"test\");\n}\n",
            false,
        );

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "Example with optional end marker:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "<!-- snips: code.rs#example -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old content").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        sync_snippets_in_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify the snippet was processed correctly
        assert!(content.contains("<!-- snips: code.rs#example -->"));
        assert!(content.contains("fn test() {"));
        assert!(content.contains("println!(\"test\");"));
    }

    #[test]
    fn mixed_end_marker_styles() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        let mut f = File::create(&code_path).unwrap();
        writeln!(f, "// snips-start: with-name").unwrap();
        writeln!(f, "fn with_name() {{}}").unwrap();
        writeln!(f, "// snips-end: with-name").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "// snips-start: without-name").unwrap();
        writeln!(f, "fn without_name() {{}}").unwrap();
        writeln!(f, "// snips-end:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "// snips-start: hyphenated-name").unwrap();
        writeln!(f, "fn hyphenated() {{}}").unwrap();
        writeln!(f, "// snips-end:").unwrap();
        drop(f);

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: code.rs#with-name -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "<!-- snips: code.rs#without-name -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "<!-- snips: code.rs#hyphenated-name -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        sync_snippets_in_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify all snippet styles work
        assert!(content.contains("fn with_name() {}"));
        assert!(content.contains("fn without_name() {}"));
        assert!(content.contains("fn hyphenated() {}"));
    }

    #[test]
    fn multiple_snippets_with_optional_ends() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        let mut f = File::create(&code_path).unwrap();
        writeln!(f, "// snips-start: first").unwrap();
        writeln!(f, "fn first() {{}}").unwrap();
        writeln!(f, "// snips-end:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "// Some other code").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "// snips-start: second").unwrap();
        writeln!(f, "fn second() {{}}").unwrap();
        writeln!(f, "// snips-end:").unwrap();
        drop(f);

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "First snippet:").unwrap();
        writeln!(f, "<!-- snips: code.rs#first -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "Second snippet:").unwrap();
        writeln!(f, "<!-- snips: code.rs#second -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        sync_snippets_in_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify both snippets work independently
        assert!(content.contains("fn first() {}"));
        assert!(content.contains("fn second() {}"));
        // Verify they don't contain each other's content
        let first_section = content.split("Second snippet:").next().unwrap();
        let second_section = content.split("Second snippet:").nth(1).unwrap();
        assert!(first_section.contains("fn first() {}"));
        assert!(!first_section.contains("fn second() {}"));
        assert!(second_section.contains("fn second() {}"));
        assert!(!second_section.contains("fn first() {}"));
    }

    #[test]
    fn optional_end_with_indentation() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_with_optional_end(
            &code_path,
            "indented-test",
            "fn indented() {\n    println!(\"indented\");\n}\n",
            false,
        );

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "1. Item with indented code:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "   <!-- snips: code.rs#indented-test -->").unwrap();
        writeln!(f, "   ```rust").unwrap();
        writeln!(f, "   old").unwrap();
        writeln!(f, "   ```").unwrap();
        drop(f);

        sync_snippets_in_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify indented snippet with optional end marker works
        assert!(content.contains("   <!-- snips: code.rs#indented-test -->"));
        assert!(content.contains("   fn indented() {"));
        assert!(content.contains("       println!(\"indented\");"));
    }

    #[test]
    fn backwards_compatibility_with_named_ends() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_with_optional_end(&code_path, "named-example", "fn named() {}\n", true);

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: code.rs#named-example -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        sync_snippets_in_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify backwards compatibility - named end markers still work
        assert!(content.contains("fn named() {}"));
    }

    // Helper to write source with colon-less end marker
    fn write_source_with_colonless_end(path: &Path, name: &str, content: &str) {
        let mut f = File::create(path).unwrap();
        writeln!(f, "// Some code before").unwrap();
        writeln!(f, "// snips-start: {name}").unwrap();
        write!(f, "{content}").unwrap();
        writeln!(f, "// snips-end").unwrap();
        writeln!(f, "// Some code after").unwrap();
    }

    #[test]
    fn colonless_end_marker() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_with_colonless_end(
            &code_path,
            "colonless-test",
            "fn colonless() {\n    println!(\"no colon\");\n}\n",
        );

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "Example with colon-less end marker:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "<!-- snips: code.rs#colonless-test -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old content").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        sync_snippets_in_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify the snippet was processed correctly
        assert!(content.contains("<!-- snips: code.rs#colonless-test -->"));
        assert!(content.contains("fn colonless() {"));
        assert!(content.contains("println!(\"no colon\");"));
    }

    #[test]
    fn mixed_colon_styles() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        let mut f = File::create(&code_path).unwrap();
        writeln!(f, "// snips-start: with-colon").unwrap();
        writeln!(f, "fn with_colon() {{}}").unwrap();
        writeln!(f, "// snips-end:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "// snips-start: without-colon").unwrap();
        writeln!(f, "fn without_colon() {{}}").unwrap();
        writeln!(f, "// snips-end").unwrap();
        drop(f);

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: code.rs#with-colon -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "<!-- snips: code.rs#without-colon -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        sync_snippets_in_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify both styles work
        assert!(content.contains("fn with_colon() {}"));
        assert!(content.contains("fn without_colon() {}"));
    }
}
