//! CLI-level integration tests.

/// Validate CLI behavior in various modes.
#[cfg(test)]
mod tests {
    #[allow(dead_code, missing_docs)]
    mod support {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/mod.rs"));
    }

    use assert_cmd::{Command, cargo::cargo_bin_cmd};
    use std::fs;
    use std::path::Path;
    use support::make_example;

    fn snips_cmd() -> Command {
        cargo_bin_cmd!("snips")
    }

    fn snips_cmd_in(path: &Path) -> Command {
        let mut cmd = snips_cmd();
        cmd.current_dir(path);
        cmd
    }

    #[test]
    fn check_fails_on_dirty_file() {
        let dir = tempfile::tempdir().unwrap();
        let md = make_example(&dir);
        snips_cmd()
            .args(["--check", md.to_str().unwrap()])
            .assert()
            .failure();
        snips_cmd().args([md.to_str().unwrap()]).assert().success();
        snips_cmd()
            .args(["--check", md.to_str().unwrap()])
            .assert()
            .success();
    }

    #[test]
    fn diff_outputs_changes() {
        let dir = tempfile::tempdir().unwrap();
        let md = make_example(&dir);
        let output = snips_cmd()
            .args(["--diff", md.to_str().unwrap()])
            .output()
            .unwrap();
        let out = String::from_utf8_lossy(&output.stdout);
        assert!(out.contains("+fn main(){}"));
    }

    #[test]
    fn render_defaults_to_markdown_files_in_cwd() {
        let dir = tempfile::tempdir().unwrap();
        let md = make_example(&dir);

        snips_cmd_in(dir.path()).assert().success();

        let content = fs::read_to_string(md).unwrap();
        assert!(!content.contains("old"));
        assert!(content.contains("fn main(){}"));
    }

    #[test]
    fn render_fails_without_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        let assert = snips_cmd_in(dir.path()).assert().failure();
        let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
        assert!(stderr.contains("no markdown files"));
    }

    #[test]
    fn check_defaults_to_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        make_example(&dir);

        snips_cmd_in(dir.path())
            .args(["--check"]) // no file args
            .assert()
            .failure();
        snips_cmd_in(dir.path()).assert().success();
        snips_cmd_in(dir.path())
            .args(["--check"]) // now clean
            .assert()
            .success();
    }

    #[test]
    fn diff_defaults_to_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        make_example(&dir);
        let output = snips_cmd_in(dir.path())
            .args(["--diff"]) // no file args
            .output()
            .unwrap();
        let out = String::from_utf8_lossy(&output.stdout);
        assert!(out.contains("+fn main(){}"));
    }
}
