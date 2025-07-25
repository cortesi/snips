use crate::error::SnipsError;
use crate::snippet::Snippet;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

static MARKER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^<!--\s*snips:\s*(?P<path>[^#\s]+)(?:#(?P<name>\w+))?\s*-->\s*$").unwrap()
});

pub fn process_file(path: &Path, write: bool) -> Result<Option<String>, SnipsError> {
    let content =
        fs::read_to_string(path).map_err(|_| SnipsError::FileNotFound(path.to_path_buf()))?;
    let base = path.parent().unwrap_or(Path::new("."));
    let processed = process_content(&content, base)?;
    if write && processed != content {
        fs::write(path, processed.clone())?;
    }
    Ok(if processed != content {
        Some(processed)
    } else {
        None
    })
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

fn process_content(content: &str, base: &Path) -> Result<String, SnipsError> {
    let marker_re = &MARKER_RE;
    let mut out = Vec::new();
    let mut lines = content.lines().enumerate();
    while let Some((idx, line)) = lines.next() {
        if line.trim_start().starts_with("<!-- snips:") {
            let caps = marker_re
                .captures(line)
                .ok_or(SnipsError::InvalidMarker(idx + 1))?;
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
                if inner.trim_start() == closing {
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
                format!("<!-- snips: {src_path}#{name} -->")
            } else {
                format!("<!-- snips: {src_path} -->")
            };
            out.push(marker);
            let lang_hint = lang.unwrap_or_default();
            if lang_hint.is_empty() {
                out.push("```".to_string());
            } else {
                out.push(format!("```{lang_hint}"));
            }
            out.push(code);
            out.push("```".to_string());
        } else {
            out.push(line.to_string());
        }
    }
    Ok(out.join("\n") + if content.ends_with('\n') { "\n" } else { "" })
}
