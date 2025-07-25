use crate::error::SnipsError;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use textwrap::dedent;

pub struct Snippet {
    pub path: PathBuf,
    pub name: Option<String>,
}

impl Snippet {
    pub fn read(&self) -> Result<(String, Option<String>), SnipsError> {
        let content = fs::read_to_string(&self.path)
            .map_err(|_| SnipsError::FileNotFound(self.path.clone()))?;
        let lang = self
            .path
            .extension()
            .and_then(|s| s.to_str())
            .and_then(|ext| languages::from_extension(ext))
            .and_then(|lang| lang.codemirror_mode)
            .map(|mode| mode.to_string());
        if let Some(name) = &self.name {
            Ok((extract_named_snippet(&content, name, &self.path)?, lang))
        } else {
            Ok((dedent(&content).to_string(), lang))
        }
    }
}

static START_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"snips-start:\s*(?P<name>\w+)\s*$").unwrap());
static END_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"snips-end:\s*(?P<name>\w+)\s*$").unwrap());

fn extract_named_snippet(content: &str, name: &str, path: &Path) -> Result<String, SnipsError> {
    let lines = content.lines();
    let mut found = false;
    let mut snippet = Vec::new();

    for line in lines {
        if !found {
            if START_RE
                .captures(line)
                .map(|c| c.name("name").map(|m| m.as_str() == name).unwrap_or(false))
                .unwrap_or(false)
            {
                found = true;
            }
            continue;
        }
        if END_RE
            .captures(line)
            .map(|c| c.name("name").map(|m| m.as_str() == name).unwrap_or(false))
            .unwrap_or(false)
        {
            let text = snippet.join("\n");
            return Ok(dedent(&text).to_string());
        }
        snippet.push(line.to_string());
    }

    if found {
        Err(SnipsError::UnterminatedSnippet(
            path.to_path_buf(),
            name.to_string(),
        ))
    } else {
        Err(SnipsError::SnippetNotFound(
            path.to_path_buf(),
            name.to_string(),
        ))
    }
}
