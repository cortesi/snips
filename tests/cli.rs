use assert_cmd::{cargo::cargo_bin_cmd, Command};
use std::fs::{self, File};
use std::io::Write;

fn snips_cmd() -> Command {
    cargo_bin_cmd!("snips")
}

fn make_example(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let code = dir.path().join("code.rs");
    fs::write(&code, "fn main(){}\n").unwrap();
    let md = dir.path().join("README.md");
    let mut f = File::create(&md).unwrap();
    writeln!(f, "<!-- snips: code.rs -->").unwrap();
    writeln!(f, "```").unwrap();
    writeln!(f, "old").unwrap();
    writeln!(f, "```").unwrap();
    md
}

#[test]
fn check_fails_on_dirty_file() {
    let dir = tempfile::tempdir().unwrap();
    let md = make_example(&dir);
    snips_cmd()
        .args(["check", md.to_str().unwrap()])
        .assert()
        .failure();
    snips_cmd()
        .args(["render", md.to_str().unwrap()])
        .assert()
        .success();
    snips_cmd()
        .args(["check", md.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn diff_outputs_changes() {
    let dir = tempfile::tempdir().unwrap();
    let md = make_example(&dir);
    let output = snips_cmd()
        .args(["diff", md.to_str().unwrap()])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("+fn main(){}"));
}

#[test]
fn render_requires_files() {
    let dir = tempfile::tempdir().unwrap();
    let _md = make_example(&dir);
    std::env::set_current_dir(&dir).unwrap();
    snips_cmd()
        .args(["render"]) // no file args
        .assert()
        .failure();
}

#[test]
fn check_requires_files() {
    let dir = tempfile::tempdir().unwrap();
    let _md = make_example(&dir);
    std::env::set_current_dir(&dir).unwrap();
    snips_cmd()
        .args(["check"]) // no file args
        .assert()
        .failure();
}

#[test]
fn diff_requires_files() {
    let dir = tempfile::tempdir().unwrap();
    let _md = make_example(&dir);
    std::env::set_current_dir(&dir).unwrap();
    snips_cmd()
        .args(["diff"]) // no file args
        .assert()
        .failure();
}
