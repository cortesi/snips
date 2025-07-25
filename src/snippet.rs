use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use once_cell::sync::Lazy;
use crate::error::SnipsError;
use textwrap::dedent;

pub struct Snippet {
    pub path: PathBuf,
    pub name: Option<String>,
}

impl Snippet {
    pub fn read(&self) -> Result<(String, Option<String>), SnipsError> {
        let content = fs::read_to_string(&self.path).map_err(|_| SnipsError::FileNotFound(self.path.clone()))?;
        let lang = self
            .path
            .extension()
            .and_then(|s| s.to_str())
            .map(lang_for_ext);
        if let Some(name) = &self.name {
            Ok((extract_named_snippet(&content, name, &self.path)?, lang.map(|l| l.to_string())))
        } else {
            Ok((dedent(&content).to_string(), lang.map(|l| l.to_string())))
        }
    }
}

static START_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"snips-start:\s*(?P<name>\w+)\s*$").unwrap()
});
static END_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"snips-end:\s*(?P<name>\w+)\s*$").unwrap()
});

fn extract_named_snippet(content: &str, name: &str, path: &Path) -> Result<String, SnipsError> {
    let mut lines = content.lines();
    let mut found = false;
    let mut snippet = Vec::new();

    while let Some(line) = lines.next() {
        if !found {
            if START_RE.captures(line).map(|c| c.name("name").map(|m| m.as_str() == name).unwrap_or(false)).unwrap_or(false) {
                found = true;
            }
            continue;
        }
        if END_RE.captures(line).map(|c| c.name("name").map(|m| m.as_str() == name).unwrap_or(false)).unwrap_or(false) {
            let text = snippet.join("\n");
            return Ok(dedent(&text).to_string());
        }
        snippet.push(line.to_string());
    }

    if found {
        Err(SnipsError::UnterminatedSnippet(path.to_path_buf(), name.to_string()))
    } else {
        Err(SnipsError::SnippetNotFound(path.to_path_buf(), name.to_string()))
    }
}

pub fn lang_for_ext(ext: &str) -> &'static str {
    match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "java" => "java",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "sh" => "bash",
        _ => "",
    }
}
