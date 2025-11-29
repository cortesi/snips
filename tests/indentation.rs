//! Integration tests covering indentation handling.

/// Verify indentation is preserved when rendering snippets.
#[cfg(test)]
mod tests {
    use snips::process_file;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    // Helper to write a source file with a snippet
    fn write_source_with_snippet(path: &Path, name: &str, content: &str) {
        let mut f = File::create(path).unwrap();
        writeln!(f, "// Some code before").unwrap();
        writeln!(f, "// snips-start: {name}").unwrap();
        write!(f, "{content}").unwrap();
        writeln!(f, "// snips-end: {name}").unwrap();
        writeln!(f, "// Some code after").unwrap();
    }

    // Helper to write a whole source file
    fn write_source_file(path: &Path, content: &str) {
        let mut f = File::create(path).unwrap();
        write!(f, "{content}").unwrap();
    }

    #[test]
    fn indented_marker_with_spaces() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_file(
            &code_path,
            "fn hello() {\n    println!(\"Hello, world!\");\n}\n",
        );

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "Some text:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "    <!-- snips: code.rs -->").unwrap();
        writeln!(f, "    ```rust").unwrap();
        writeln!(f, "    old content").unwrap();
        writeln!(f, "    ```").unwrap();
        drop(f);

        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify the marker is indented
        assert!(content.contains("    <!-- snips: code.rs -->"));
        // Verify the code fences are indented
        assert!(content.contains("    ```rust"));
        assert!(content.contains("    ```\n"));
        // Verify the code content is indented
        assert!(content.contains("    fn hello() {"));
        assert!(content.contains("        println!(\"Hello, world!\");"));
        assert!(content.contains("    }"));
    }

    #[test]
    fn indented_marker_with_tabs() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_file(&code_path, "fn hello() {\n    println!(\"Hello!\");\n}\n");

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "Some text:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "\t<!-- snips: code.rs -->").unwrap();
        writeln!(f, "\t```rust").unwrap();
        writeln!(f, "\told content").unwrap();
        writeln!(f, "\t```").unwrap();
        drop(f);

        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify the marker is indented with tab
        assert!(content.contains("\t<!-- snips: code.rs -->"));
        // Verify the code fences are indented with tab
        assert!(content.contains("\t```rust"));
        assert!(content.contains("\t```\n"));
        // Verify the code content is indented with tab
        assert!(content.contains("\tfn hello() {"));
        assert!(content.contains("\t    println!(\"Hello!\");"));
        assert!(content.contains("\t}"));
    }

    #[test]
    fn indented_named_snippet() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_with_snippet(
            &code_path,
            "example",
            "fn test() {\n    println!(\"test\");\n}\n",
        );

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "1. First item").unwrap();
        writeln!(f, "2. Second item with code:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "   <!-- snips: code.rs#example -->").unwrap();
        writeln!(f, "   ```rust").unwrap();
        writeln!(f, "   old content").unwrap();
        writeln!(f, "   ```").unwrap();
        drop(f);

        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify the marker is indented
        assert!(content.contains("   <!-- snips: code.rs#example -->"));
        // Verify the code fences are indented
        assert!(content.contains("   ```rust"));
        assert!(content.contains("   ```\n"));
        // Verify the code content is indented (3 spaces from original + content indentation)
        assert!(content.contains("   fn test() {"));
        assert!(content.contains("       println!(\"test\");"));
        assert!(content.contains("   }"));
    }

    #[test]
    fn mixed_indentation_levels() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_file(&code_path, "fn simple() {}\n");

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "No indentation:").unwrap();
        writeln!(f, "<!-- snips: code.rs -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "With indentation:").unwrap();
        writeln!(f, "  <!-- snips: code.rs -->").unwrap();
        writeln!(f, "  ```rust").unwrap();
        writeln!(f, "  old").unwrap();
        writeln!(f, "  ```").unwrap();
        drop(f);

        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // First block should have no indentation
        assert!(content.contains("<!-- snips: code.rs -->"));
        assert!(content.contains("```rust\nfn simple() {}\n\n```"));

        // Second block should be indented with 2 spaces
        assert!(content.contains("  <!-- snips: code.rs -->"));
        assert!(content.contains("  ```rust\n  fn simple() {}\n  ```"));
    }

    #[test]
    fn preserve_empty_lines_in_indented_code() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_file(
            &code_path,
            "fn test() {\n    let x = 1;\n\n    let y = 2;\n}\n",
        );

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "Example:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "    <!-- snips: code.rs -->").unwrap();
        writeln!(f, "    ```rust").unwrap();
        writeln!(f, "    old").unwrap();
        writeln!(f, "    ```").unwrap();
        drop(f);

        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify empty lines are preserved without adding indentation
        let lines: Vec<&str> = content.lines().collect();
        // Look for the correctly indented line (original 4 space indent + content indentation)
        let empty_line_idx = lines
            .iter()
            .position(|&line| line == "        let x = 1;")
            .unwrap()
            + 1;
        assert_eq!(lines[empty_line_idx], ""); // Empty line should remain empty
        assert!(content.contains("        let x = 1;\n\n        let y = 2;"));
    }

    #[test]
    fn deeply_indented_marker() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_file(&code_path, "const X: i32 = 42;\n");

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "- Item 1").unwrap();
        writeln!(f, "  - Nested item").unwrap();
        writeln!(f, "    - Deep nested with code:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "      <!-- snips: code.rs -->").unwrap();
        writeln!(f, "      ```rust").unwrap();
        writeln!(f, "      old").unwrap();
        writeln!(f, "      ```").unwrap();
        drop(f);

        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify deep indentation is preserved
        assert!(content.contains("      <!-- snips: code.rs -->"));
        assert!(content.contains("      ```rust"));
        assert!(content.contains("      const X: i32 = 42;"));
        assert!(content.contains("      ```"));
    }

    #[test]
    fn zero_indentation_marker() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        write_source_file(&code_path, "fn main() {}\n");

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: code.rs -->").unwrap();
        writeln!(f, "```rust").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        process_file(&md_path, true).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();

        // Verify no extra indentation is added when marker has no indentation
        assert!(content.contains("<!-- snips: code.rs -->"));
        assert!(content.contains("```rust\nfn main() {}\n\n```"));
    }
}
