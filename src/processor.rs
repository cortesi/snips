use crate::error::SnipsError;
use crate::snippet::{SNIPPET_ID_CHARS, Snippet};
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

/// A difference between existing markdown content and the current snippet content.
#[derive(Debug)]
pub struct SnippetDiff {
    /// Snippet source path relative to the markdown file.
    pub path: String,
    /// Optional snippet name inside the source file.
    pub name: Option<String>,
    /// Content currently present in the markdown file.
    pub old_content: String,
    /// Fresh content read from the source file.
    pub new_content: String,
}

/// Regex that matches a `<!-- snips: ... -->` marker and captures indentation,
/// source path, and optional snippet name.
static MARKER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"^(?P<indent>\s*)<!--\s*snips:\s*(?P<path>[^#\s]+)(?:#(?P<name>{SNIPPET_ID_CHARS}+))?\s*-->\s*$"
    ))
    .unwrap()
});

/// Apply indentation to every non-blank line in `content`.
fn apply_indentation(content: &str, indent: &str) -> String {
    if indent.is_empty() {
        content.to_string()
    } else {
        content
            .lines()
            .map(|line| {
                if line.trim().is_empty() {
                    line.to_string()
                } else {
                    format!("{indent}{line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Process a single markdown file and optionally write updates in place.
pub fn process_file(path: &Path, write: bool) -> Result<Option<String>, SnipsError> {
    let content =
        fs::read_to_string(path).map_err(|_| SnipsError::FileNotFound(path.to_path_buf()))?;
    let base = path.parent().unwrap_or(Path::new("."));
    let processed = process_content(&content, base, path)?;
    if write && processed != content {
        fs::write(path, processed.clone())?;
    }
    Ok(if processed != content {
        Some(processed)
    } else {
        None
    })
}

/// Compute diffs between snippets embedded in `path` and their sources.
pub fn get_snippet_diffs(path: &Path) -> Result<Vec<SnippetDiff>, SnipsError> {
    let content =
        fs::read_to_string(path).map_err(|_| SnipsError::FileNotFound(path.to_path_buf()))?;
    let base = path.parent().unwrap_or(Path::new("."));
    get_content_diffs(&content, base, path)
}

/// Marker type used to expose the `check` entry point without state.
pub struct Processor;

impl Processor {
    /// Return `true` when all provided markdown files are already up to date.
    pub fn check(paths: &[PathBuf]) -> Result<bool, SnipsError> {
        let mut clean = true;
        for p in paths {
            if process_file(p, false)?.is_some() {
                clean = false;
            }
        }
        Ok(clean)
    }
}

/// Scan markdown content for snippet markers and compute diffs against source files.
fn get_content_diffs(
    content: &str,
    base: &Path,
    file_path: &Path,
) -> Result<Vec<SnippetDiff>, SnipsError> {
    let marker_re = &MARKER_RE;
    let mut diffs = Vec::new();
    let mut lines = content.lines().enumerate();

    while let Some((idx, line)) = lines.next() {
        if line.trim_start().starts_with("<!-- snips:") {
            let caps = marker_re.captures(line).ok_or(SnipsError::InvalidMarker {
                file: file_path.to_path_buf(),
                line: idx + 1,
                content: line.to_string(),
            })?;
            let indent = caps.name("indent").unwrap().as_str();
            let src_path = caps.name("path").unwrap().as_str();
            let snippet_name = caps.name("name").map(|m| m.as_str().to_string());

            let (_, fence_line) = lines.next().ok_or(SnipsError::MissingCodeFence(idx + 1))?;
            let trimmed = fence_line.trim_start();
            if !trimmed.starts_with("```") {
                return Err(SnipsError::MissingCodeFence(idx + 1));
            }
            let tick_count = trimmed.chars().take_while(|&c| c == '`').count();
            let closing = "`".repeat(tick_count);

            let mut old_content_lines = Vec::new();
            for (_, inner) in lines.by_ref() {
                if inner.trim() == closing {
                    break;
                }
                old_content_lines.push(inner.to_string());
            }
            let old_content = old_content_lines.join("\n");

            let target = base.join(src_path);
            let snippet = Snippet {
                path: target,
                name: snippet_name.clone(),
            };
            let (new_content, _) = snippet.read()?;

            // Apply the same indentation to new_content as process_content does
            let new_content_with_indent = apply_indentation(&new_content, indent);

            if old_content.trim() != new_content_with_indent.trim() {
                diffs.push(SnippetDiff {
                    path: src_path.to_string(),
                    name: snippet_name,
                    old_content,
                    new_content: new_content_with_indent,
                });
            }
        }
    }
    Ok(diffs)
}

/// Replace every snippet marker in `content` with the latest snippet text.
fn process_content(content: &str, base: &Path, file_path: &Path) -> Result<String, SnipsError> {
    let marker_re = &MARKER_RE;
    let mut out = Vec::new();
    let mut lines = content.lines().enumerate();
    while let Some((idx, line)) = lines.next() {
        if line.trim_start().starts_with("<!-- snips:") {
            let caps = marker_re.captures(line).ok_or(SnipsError::InvalidMarker {
                file: file_path.to_path_buf(),
                line: idx + 1,
                content: line.to_string(),
            })?;
            let indent = caps.name("indent").unwrap().as_str();
            let src_path = caps.name("path").unwrap().as_str();
            let snippet_name = caps.name("name").map(|m| m.as_str().to_string());

            let (_, fence_line) = lines.next().ok_or(SnipsError::MissingCodeFence(idx + 1))?;
            let trimmed = fence_line.trim_start();
            if !trimmed.starts_with("```") {
                return Err(SnipsError::MissingCodeFence(idx + 1));
            }
            let tick_count = trimmed.chars().take_while(|&c| c == '`').count();
            let closing = "`".repeat(tick_count);

            for (_, inner) in lines.by_ref() {
                if inner.trim() == closing {
                    break;
                }
            }
            let target = base.join(src_path);
            let snippet = Snippet {
                path: target,
                name: snippet_name.clone(),
            };
            let (code, lang) = snippet.read()?;
            let marker = if let Some(name) = &snippet_name {
                format!("{indent}<!-- snips: {src_path}#{name} -->")
            } else {
                format!("{indent}<!-- snips: {src_path} -->")
            };
            out.push(marker);
            let lang_hint = lang.unwrap_or_default();
            if lang_hint.is_empty() {
                out.push(format!("{indent}```"));
            } else {
                out.push(format!("{indent}```{lang_hint}"));
            }
            out.push(apply_indentation(&code, indent));
            out.push(format!("{indent}```"));
        } else {
            out.push(line.to_string());
        }
    }
    Ok(out.join("\n") + if content.ends_with('\n') { "\n" } else { "" })
}
