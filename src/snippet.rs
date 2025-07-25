use crate::error::SnipsError;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use textwrap::dedent;

// Valid characters for snippet identifiers: letters, digits, underscores, and hyphens
const SNIPPET_ID_CHARS: &str = r"[\w-]";

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

static START_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"snips-start:\s*(?P<name>{SNIPPET_ID_CHARS}+)\s*$"
    ))
    .unwrap()
});
static END_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"snips-end(?::(?:\s*(?P<name>{SNIPPET_ID_CHARS}+))?)?\s*$"
    ))
    .unwrap()
});

fn find_available_snippets(content: &str) -> Vec<String> {
    let mut snippets = Vec::new();
    for line in content.lines() {
        if let Some(caps) = START_RE.captures(line)
            && let Some(name) = caps.name("name")
        {
            snippets.push(name.as_str().to_string());
        }
    }
    snippets
}

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
        if END_RE.captures(line).is_some_and(|c| {
            // End marker matches if it has no name or if the name matches our target
            c.name("name").is_none_or(|m| m.as_str() == name)
        }) {
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
        let available = find_available_snippets(content);
        let available_display = if available.is_empty() {
            "none".to_string()
        } else {
            available.join(", ")
        };
        Err(SnipsError::SnippetNotFound {
            file: path.to_path_buf(),
            snippet_name: name.to_string(),
            available_snippets: available_display,
        })
    }
}
