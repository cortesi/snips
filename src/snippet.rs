use crate::error::SnipsError;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use textwrap::dedent;

/// Allowed characters for snippet identifiers.
pub(crate) const SNIPPET_ID_CHARS: &str = r"[\w-]";

/// A snippet reference made up of a source path and an optional named section.
pub(crate) struct SnippetRef {
    /// Path to the source file that contains the snippet.
    pub path: PathBuf,
    /// Name of the snippet within the file, if one is specified.
    pub name: Option<String>,
}

impl SnippetRef {
    /// Read the referenced snippet content and infer a language hint.
    ///
    /// When `name` is `None`, the whole file is returned. Otherwise the
    /// named section between `snips-start`/`snips-end` markers is extracted.
    pub fn resolve(&self) -> Result<(String, Option<String>), SnipsError> {
        let content = fs::read_to_string(&self.path).map_err(|source| match source.kind() {
            ErrorKind::NotFound => SnipsError::FileNotFound {
                file: self.path.clone(),
                source,
            },
            _ => SnipsError::FileReadFailed {
                file: self.path.clone(),
                source,
            },
        })?;
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
            Ok((dedent(&content), lang))
        }
    }
}

/// Matches a `snips-start` marker and captures the snippet name.
static START_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"snips-start:\s*(?P<name>{SNIPPET_ID_CHARS}+)\s*$"
    ))
    .unwrap()
});
/// Matches a `snips-end` marker with an optional snippet name.
static END_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"snips-end(?::(?:\s*(?P<name>{SNIPPET_ID_CHARS}+))?)?\s*$"
    ))
    .unwrap()
});

/// Collect the names of all snippets available in the provided content.
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

/// Extract a named snippet between matching start/end markers, respecting indentation.
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
            return Ok(dedent(&text));
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
