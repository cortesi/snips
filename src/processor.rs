use crate::error::SnipsError;
use crate::snippet::Snippet;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct SnippetDiff {
    pub path: String,
    pub name: Option<String>,
    pub old_content: String,
    pub new_content: String,
}

static MARKER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<indent>\s*)<!--\s*snips:\s*(?P<path>[^#\s]+)(?:#(?P<name>\w+))?\s*-->\s*$").unwrap()
});

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

pub fn get_snippet_diffs(path: &Path) -> Result<Vec<SnippetDiff>, SnipsError> {
    let content =
        fs::read_to_string(path).map_err(|_| SnipsError::FileNotFound(path.to_path_buf()))?;
    let base = path.parent().unwrap_or(Path::new("."));
    get_content_diffs(&content, base, path)
}

pub struct Processor;

impl Processor {
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

fn get_content_diffs(content: &str, base: &Path, file_path: &Path) -> Result<Vec<SnippetDiff>, SnipsError> {
    let marker_re = &MARKER_RE;
    let mut diffs = Vec::new();
    let mut lines = content.lines().enumerate();

    while let Some((idx, line)) = lines.next() {
        if line.trim_start().starts_with("<!-- snips:") {
            let caps = marker_re
                .captures(line)
                .ok_or(SnipsError::InvalidMarker {
                    file: file_path.to_path_buf(),
                    line: idx + 1,
                    content: line.to_string(),
                })?;
            let _indent = caps.name("indent").unwrap().as_str();
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

            if old_content.trim() != new_content.trim() {
                diffs.push(SnippetDiff {
                    path: src_path.to_string(),
                    name: snippet_name,
                    old_content,
                    new_content,
                });
            }
        }
    }
    Ok(diffs)
}

fn process_content(content: &str, base: &Path, file_path: &Path) -> Result<String, SnipsError> {
    let marker_re = &MARKER_RE;
    let mut out = Vec::new();
    let mut lines = content.lines().enumerate();
    while let Some((idx, line)) = lines.next() {
        if line.trim_start().starts_with("<!-- snips:") {
            let caps = marker_re
                .captures(line)
                .ok_or(SnipsError::InvalidMarker {
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
