//! Integration tests for command-driven and header-scoped markers.

/// Validate command markers and header replacements.
#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
    };

    use snips::{CommandPolicy, SnipsError, sync_snippets_in_file};

    #[test]
    fn header_scoped_replacement_updates_section() {
        let dir = tempfile::tempdir().unwrap();
        let code_path = dir.path().join("code.rs");
        fs::write(
            &code_path,
            "// snips-start: docs\nLine A\nLine B\n// snips-end: docs\n",
        )
        .unwrap();

        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: code.rs#docs -->").unwrap();
        writeln!(f, "# API Documentation").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "Old text").unwrap();
        writeln!(f, "## Subheader").unwrap();
        writeln!(f, "Old subtext").unwrap();
        writeln!(f, "# Next").unwrap();
        writeln!(f, "Keep").unwrap();
        drop(f);

        sync_snippets_in_file(&md_path, true, CommandPolicy::Deny).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();
        assert!(content.contains("# API Documentation"));
        assert!(content.contains("Line A"));
        assert!(!content.contains("Old text"));
        assert!(!content.contains("## Subheader"));
        assert!(content.contains("# Next"));
        assert!(content.contains("Keep"));
    }

    #[test]
    fn command_marker_replaces_header_section() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: !cargo --version -->").unwrap();
        writeln!(f, "# CLI Help").unwrap();
        writeln!(f, "Old").unwrap();
        drop(f);

        sync_snippets_in_file(&md_path, true, CommandPolicy::Allow).unwrap();
        let content = fs::read_to_string(&md_path).unwrap();
        assert!(content.contains("# CLI Help"));
        assert!(content.contains("cargo "));
        assert!(!content.contains("Old"));
    }

    #[test]
    fn command_policy_deny_blocks_execution() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: !cargo --version -->").unwrap();
        writeln!(f, "# CLI Help").unwrap();
        writeln!(f, "Old").unwrap();
        drop(f);

        match sync_snippets_in_file(&md_path, false, CommandPolicy::Deny) {
            Err(SnipsError::CommandExecutionDisabled { command }) => {
                assert_eq!(command, "cargo --version");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn command_failure_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("doc.md");
        let mut f = File::create(&md_path).unwrap();
        writeln!(f, "<!-- snips: !cargo no-such-subcommand -->").unwrap();
        writeln!(f, "```").unwrap();
        writeln!(f, "old").unwrap();
        writeln!(f, "```").unwrap();
        drop(f);

        match sync_snippets_in_file(&md_path, false, CommandPolicy::Allow) {
            Err(SnipsError::CommandFailed { command, .. }) => {
                assert_eq!(command, "cargo no-such-subcommand");
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
