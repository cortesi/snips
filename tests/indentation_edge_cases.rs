//! Integration tests for indentation edge cases.

/// Stress tests for nested and mixed indentation scenarios.
#[cfg(test)]
mod tests {
    use snips::{diff_file, sync_snippets_in_file};
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    // Helper to write a source file with indented content
    fn write_indented_source(path: &Path, content: &str) {
        let mut f = File::create(path).unwrap();
        write!(f, "{content}").unwrap();
    }

    // Helper to write markdown with specific indentation
    fn write_indented_markdown(path: &Path, indent: &str, snippet_path: &str, code_content: &str) {
        let mut f = File::create(path).unwrap();
        writeln!(f, "Some documentation:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "{indent}<!-- snips: {snippet_path} -->").unwrap();
        writeln!(f, "{indent}```rust").unwrap();
        for line in code_content.lines() {
            if line.trim().is_empty() {
                writeln!(f).unwrap();
            } else {
                writeln!(f, "{indent}{line}").unwrap();
            }
        }
        writeln!(f, "{indent}```").unwrap();
    }

    #[test]
    fn indented_source_with_indented_markdown_no_diff() {
        // This test reproduces the bug where render makes no changes
        // but diff still shows differences
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");

        // Source file with indented content
        write_indented_source(
            &code_path,
            "fn main() {\n    if true {\n        println!(\"hello\");\n    }\n}\n",
        );

        let md_path = dir.path().join("doc.md");

        // Markdown with 4-space indentation containing the correctly indented content
        write_indented_markdown(
            &md_path,
            "    ",
            "code.rs",
            "fn main() {\n    if true {\n        println!(\"hello\");\n    }\n}",
        );

        // First, check if render makes any changes
        let render_result = sync_snippets_in_file(&md_path, false).unwrap();
        println!("Render result (should be None): {:?}", render_result);

        // Check what diff reports
        let diffs = diff_file(&md_path).unwrap();
        println!("Number of diffs found: {}", diffs.len());

        for diff in &diffs {
            println!("Diff found:");
            println!("  Path: {}", diff.path);
            println!("  Old content: {:?}", diff.old_content);
            println!("  New content: {:?}", diff.new_content);
            println!("  Old trimmed: {:?}", diff.old_content.trim());
            println!("  New trimmed: {:?}", diff.new_content.trim());
        }

        // This should pass but currently might fail due to the bug
        assert!(
            render_result.is_none(),
            "Render should make no changes when content matches"
        );
        assert!(
            diffs.is_empty(),
            "Diff should report no changes when content matches"
        );
    }

    #[test]
    fn various_indentation_combinations() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");

        // Source with mixed indentation
        write_indented_source(
            &code_path,
            "struct Example {\n    field: i32,\n}\n\nimpl Example {\n    fn method(&self) {\n        println!(\"value: {}\", self.field);\n    }\n}\n",
        );

        let test_cases = vec![
            ("no_indent", ""),
            ("two_spaces", "  "),
            ("four_spaces", "    "),
            ("tab", "\t"),
            ("mixed", "  \t"),
        ];

        for (name, indent) in test_cases {
            let md_path = dir.path().join(format!("doc_{name}.md"));

            // Write markdown with the current indentation level
            write_indented_markdown(
                &md_path,
                indent,
                "code.rs",
                "struct Example {\n    field: i32,\n}\n\nimpl Example {\n    fn method(&self) {\n        println!(\"value: {}\", self.field);\n    }\n}",
            );

            // Test render consistency
            let render_result1 = sync_snippets_in_file(&md_path, false).unwrap();
            let render_result2 = sync_snippets_in_file(&md_path, false).unwrap();

            assert_eq!(
                render_result1, render_result2,
                "Render should be idempotent for {name} indentation"
            );

            // Test diff consistency
            let diffs1 = diff_file(&md_path).unwrap();
            let diffs2 = diff_file(&md_path).unwrap();

            assert_eq!(
                diffs1.len(),
                diffs2.len(),
                "Diff should be consistent for {name} indentation"
            );

            // If render makes no changes, diff should report no changes
            if render_result1.is_none() {
                assert!(
                    diffs1.is_empty(),
                    "If render makes no changes, diff should report no changes for {name} indentation"
                );
            }
        }
    }

    #[test]
    fn deep_indentation_with_nested_content() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");

        // Source with deeply nested content
        write_indented_source(
            &code_path,
            "mod outer {\n    mod inner {\n        fn deeply_nested() {\n            if let Some(value) = option {\n                match value {\n                    1 => println!(\"one\"),\n                    _ => println!(\"other\"),\n                }\n            }\n        }\n    }\n}\n",
        );

        let md_path = dir.path().join("doc.md");

        // Markdown with 8-space indentation (deeply nested list)
        write_indented_markdown(
            &md_path,
            "        ",
            "code.rs",
            "mod outer {\n    mod inner {\n        fn deeply_nested() {\n            if let Some(value) = option {\n                match value {\n                    1 => println!(\"one\"),\n                    _ => println!(\"other\"),\n                }\n            }\n        }\n    }\n}",
        );

        // Test multiple render passes
        let content_before = fs::read_to_string(&md_path).unwrap();

        let render1 = sync_snippets_in_file(&md_path, false).unwrap();
        let render2 = sync_snippets_in_file(&md_path, false).unwrap();
        let render3 = sync_snippets_in_file(&md_path, false).unwrap();

        let content_after = fs::read_to_string(&md_path).unwrap();

        // Content should not change between renders
        assert_eq!(
            content_before, content_after,
            "Content should not change during multiple renders"
        );

        // All renders should agree
        assert_eq!(render1, render2, "First and second render should agree");
        assert_eq!(render2, render3, "Second and third render should agree");

        // Test diff consistency
        let diffs = diff_file(&md_path).unwrap();

        if render1.is_none() {
            assert!(
                diffs.is_empty(),
                "No diffs should be reported when render makes no changes"
            );
        }
    }

    #[test]
    fn empty_lines_and_trailing_whitespace() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");

        // Source with empty lines and various whitespace
        write_indented_source(
            &code_path,
            "fn function_with_gaps() {\n    let x = 1;\n\n    let y = 2;\n\n\n    println!(\"result: {}\", x + y);\n}\n",
        );

        let md_path = dir.path().join("doc.md");

        // Markdown with correct indentation including empty lines
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "Example:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "    <!-- snips: code.rs -->").unwrap();
        writeln!(f, "    ```rust").unwrap();
        writeln!(f, "    fn function_with_gaps() {{").unwrap();
        writeln!(f, "        let x = 1;").unwrap();
        writeln!(f).unwrap(); // Empty line
        writeln!(f, "        let y = 2;").unwrap();
        writeln!(f).unwrap(); // Empty line
        writeln!(f).unwrap(); // Empty line
        writeln!(f, "        println!(\"result: {{}}\", x + y);").unwrap();
        writeln!(f, "    }}").unwrap();
        writeln!(f, "    ```").unwrap();
        drop(f);

        // Test render and diff consistency
        let render_result = sync_snippets_in_file(&md_path, false).unwrap();
        let diffs = diff_file(&md_path).unwrap();

        println!("Empty lines test - render result: {:?}", render_result);
        println!("Empty lines test - diffs: {}", diffs.len());

        if render_result.is_none() {
            assert!(
                diffs.is_empty(),
                "No diffs when render makes no changes (empty lines test)"
            );
        }
    }

    #[test]
    fn tabs_vs_spaces_consistency() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");

        // Source using spaces for indentation
        write_indented_source(
            &code_path,
            "fn spaces() {\n    println!(\"using spaces\");\n}\n",
        );

        // Test with markdown using tabs
        let md_path_tabs = dir.path().join("doc_tabs.md");
        write_indented_markdown(
            &md_path_tabs,
            "\t",
            "code.rs",
            "fn spaces() {\n    println!(\"using spaces\");\n}",
        );

        // Test with markdown using spaces
        let md_path_spaces = dir.path().join("doc_spaces.md");
        write_indented_markdown(
            &md_path_spaces,
            "    ",
            "code.rs",
            "fn spaces() {\n    println!(\"using spaces\");\n}",
        );

        for (name, path) in [("tabs", &md_path_tabs), ("spaces", &md_path_spaces)] {
            let render_result = sync_snippets_in_file(path, false).unwrap();
            let diffs = diff_file(path).unwrap();

            println!("{name} test - render result: {:?}", render_result);
            println!("{name} test - diffs: {}", diffs.len());

            if render_result.is_none() {
                assert!(
                    diffs.is_empty(),
                    "No diffs when render makes no changes ({name} test)"
                );
            }
        }
    }

    #[test]
    fn render_diff_consistency_complex() {
        // Test that render and diff always agree on complex nested structures
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");

        write_indented_source(
            &code_path,
            "mod test {\n    use std::collections::HashMap;\n\n    pub struct Config {\n        map: HashMap<String, String>,\n    }\n\n    impl Config {\n        pub fn new() -> Self {\n            Self {\n                map: HashMap::new(),\n            }\n        }\n\n        pub fn set(&mut self, key: String, value: String) {\n            self.map.insert(key, value);\n        }\n    }\n}\n",
        );

        let md_path = dir.path().join("doc.md");
        write_indented_markdown(
            &md_path,
            "      ",
            "code.rs",
            "mod test {\n    use std::collections::HashMap;\n\n    pub struct Config {\n        map: HashMap<String, String>,\n    }\n\n    impl Config {\n        pub fn new() -> Self {\n            Self {\n                map: HashMap::new(),\n            }\n        }\n\n        pub fn set(&mut self, key: String, value: String) {\n            self.map.insert(key, value);\n        }\n    }\n}",
        );

        // Test multiple iterations to ensure stability
        for i in 1..=5 {
            let render_result = sync_snippets_in_file(&md_path, false).unwrap();
            let diffs = diff_file(&md_path).unwrap();

            if render_result.is_none() {
                assert!(
                    diffs.is_empty(),
                    "Iteration {i}: No diffs when render makes no changes"
                );
            } else {
                assert!(
                    !diffs.is_empty(),
                    "Iteration {i}: Diffs should exist when render makes changes"
                );
            }
        }
    }

    #[test]
    fn named_snippets_with_indentation() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");

        let mut f = File::create(&code_path).unwrap();
        writeln!(f, "// snips-start: function-a").unwrap();
        writeln!(f, "fn function_a() {{").unwrap();
        writeln!(f, "    println!(\"A\");").unwrap();
        writeln!(f, "}}").unwrap();
        writeln!(f, "// snips-end: function-a").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "// snips-start: function-b").unwrap();
        writeln!(f, "fn function_b() {{").unwrap();
        writeln!(f, "    println!(\"B\");").unwrap();
        writeln!(f, "}}").unwrap();
        writeln!(f, "// snips-end: function-b").unwrap();
        drop(f);

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "Functions:").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "  <!-- snips: code.rs#function-a -->").unwrap();
        writeln!(f, "  ```rust").unwrap();
        writeln!(f, "  fn function_a() {{").unwrap();
        writeln!(f, "      println!(\"A\");").unwrap();
        writeln!(f, "  }}").unwrap();
        writeln!(f, "  ```").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "    <!-- snips: code.rs#function-b -->").unwrap();
        writeln!(f, "    ```rust").unwrap();
        writeln!(f, "    fn function_b() {{").unwrap();
        writeln!(f, "        println!(\"B\");").unwrap();
        writeln!(f, "    }}").unwrap();
        writeln!(f, "    ```").unwrap();
        drop(f);

        // Test render/diff consistency
        let render_result = sync_snippets_in_file(&md_path, false).unwrap();
        let diffs = diff_file(&md_path).unwrap();

        if render_result.is_none() {
            assert!(
                diffs.is_empty(),
                "Named snippets: No diffs when render makes no changes"
            );
        } else {
            assert!(
                !diffs.is_empty(),
                "Named snippets: Diffs should exist when render makes changes"
            );
        }
    }
}
