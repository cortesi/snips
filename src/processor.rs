use crate::error::SnipsError;
use crate::snippet::{SNIPPET_ID_CHARS, SnippetRef};
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::iter::Enumerate;
use std::path::{Path, PathBuf};
use std::str::Lines;

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

/// A snippet reference captured from a markdown file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnippetLocator {
    /// Snippet source path relative to the markdown file.
    pub path: String,
    /// Optional snippet name inside the source file.
    pub name: Option<String>,
}

impl SnippetLocator {
    /// Render the locator in marker form (e.g., `path/to/file#name`).
    pub fn marker(&self) -> String {
        match &self.name {
            Some(name) => format!("{}#{name}", self.path),
            None => self.path.clone(),
        }
    }
}

/// Per-snippet report for a rendered markdown file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnippetReport {
    /// Snippet location details.
    pub locator: SnippetLocator,
    /// Whether this snippet's content changed during render.
    pub updated: bool,
}

/// Result of rendering snippets within a single markdown file.
#[derive(Debug)]
pub struct RenderSummary {
    /// Whether the file content changed during rendering.
    pub updated: bool,
    /// The rendered content when `updated` is `true`.
    pub rendered: Option<String>,
    /// Snippet references found while processing the file.
    pub snippets: Vec<SnippetReport>,
}

/// Regex that matches a `<!-- snips: ... -->` marker and captures indentation,
/// source path, and optional snippet name.
static MARKER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"^(?P<indent>\s*)<!--\s*snips:\s*(?P<path>[^#\s]+)(?:#(?P<name>{SNIPPET_ID_CHARS}+))?\s*-->\s*$"
    ))
    .unwrap()
});

/// Parsed representation of a snippet marker and its fenced content.
struct ParsedSnippet {
    /// Whitespace indentation preceding the marker.
    indent: String,
    /// Source information recovered from the marker line.
    locator: SnippetLocator,
    /// Original snippet text found between fences.
    old_content: String,
}

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
pub fn sync_snippets_in_file(path: &Path, write: bool) -> Result<Option<String>, SnipsError> {
    Ok(sync_snippets_in_file_with_summary(path, write)?.rendered)
}

/// Process a single markdown file, returning snippet metadata alongside changes.
pub fn sync_snippets_in_file_with_summary(
    path: &Path,
    write: bool,
) -> Result<RenderSummary, SnipsError> {
    let content =
        fs::read_to_string(path).map_err(|_| SnipsError::FileNotFound(path.to_path_buf()))?;
    let base = path.parent().unwrap_or(Path::new("."));
    let injection = inject_snippet_content(&content, base, path)?;
    let updated = injection.rendered != content;
    if write && updated {
        fs::write(path, injection.rendered.clone())?;
    }

    Ok(RenderSummary {
        updated,
        rendered: updated.then_some(injection.rendered),
        snippets: injection.snippets,
    })
}

/// Compute diffs between snippets embedded in `path` and their sources.
pub fn diff_file(path: &Path) -> Result<Vec<SnippetDiff>, SnipsError> {
    let content =
        fs::read_to_string(path).map_err(|_| SnipsError::FileNotFound(path.to_path_buf()))?;
    let base = path.parent().unwrap_or(Path::new("."));
    compute_diffs(&content, base, path)
}

/// Scan a list of files and return `true` if they are all up to date.
pub fn check_files(paths: &[PathBuf]) -> Result<bool, SnipsError> {
    let mut clean = true;
    for p in paths {
        if sync_snippets_in_file(p, false)?.is_some() {
            clean = false;
        }
    }
    Ok(clean)
}

/// Scan markdown content for snippet markers and compute diffs against source files.
fn compute_diffs(
    content: &str,
    base: &Path,
    file_path: &Path,
) -> Result<Vec<SnippetDiff>, SnipsError> {
    let marker_re = &MARKER_RE;
    let mut diffs = Vec::new();
    let mut lines = content.lines().enumerate();

    while let Some((idx, line)) = lines.next() {
        if line.trim_start().starts_with("<!-- snips:") {
            let parsed = parse_snippet_block(marker_re, file_path, idx, line, &mut lines)?;
            let target = base.join(&parsed.locator.path);
            let snippet = SnippetRef {
                path: target,
                name: parsed.locator.name.clone(),
            };
            let (new_content, _) = snippet.resolve()?;

            // Apply the same indentation to new_content as process_content does
            let new_content_with_indent = apply_indentation(&new_content, &parsed.indent);

            if parsed.old_content.trim() != new_content_with_indent.trim() {
                diffs.push(SnippetDiff {
                    path: parsed.locator.path,
                    name: parsed.locator.name,
                    old_content: parsed.old_content,
                    new_content: new_content_with_indent,
                });
            }
        }
    }
    Ok(diffs)
}

/// Replace every snippet marker in `content` with the latest snippet text.
fn inject_snippet_content(
    content: &str,
    base: &Path,
    file_path: &Path,
) -> Result<InjectionResult, SnipsError> {
    let marker_re = &MARKER_RE;
    let mut out = Vec::new();
    let mut snippets = Vec::new();
    let mut lines = content.lines().enumerate();
    while let Some((idx, line)) = lines.next() {
        if line.trim_start().starts_with("<!-- snips:") {
            let parsed = parse_snippet_block(marker_re, file_path, idx, line, &mut lines)?;
            let target = base.join(&parsed.locator.path);
            let snippet = SnippetRef {
                path: target,
                name: parsed.locator.name.clone(),
            };
            let (code, lang) = snippet.resolve()?;
            let indent = parsed.indent.as_str();
            let marker = if let Some(name) = &parsed.locator.name {
                format!("{indent}<!-- snips: {}#{name} -->", parsed.locator.path)
            } else {
                format!("{indent}<!-- snips: {} -->", parsed.locator.path)
            };
            out.push(marker);
            let lang_hint = lang.unwrap_or_default();
            if lang_hint.is_empty() {
                out.push(format!("{indent}```"));
            } else {
                out.push(format!("{indent}```{lang_hint}"));
            }
            let rendered_snippet = apply_indentation(&code, indent);
            let updated = parsed.old_content.trim() != rendered_snippet.trim();
            snippets.push(SnippetReport {
                locator: parsed.locator.clone(),
                updated,
            });
            out.push(rendered_snippet);
            out.push(format!("{indent}```"));
        } else {
            out.push(line.to_string());
        }
    }
    Ok(InjectionResult {
        rendered: out.join("\n") + if content.ends_with('\n') { "\n" } else { "" },
        snippets,
    })
}

/// Consume a marker line and its fenced block, returning parsed details.
fn parse_snippet_block(
    marker_re: &Regex,
    file_path: &Path,
    idx: usize,
    line: &str,
    lines: &mut Enumerate<Lines<'_>>,
) -> Result<ParsedSnippet, SnipsError> {
    let caps = marker_re.captures(line).ok_or(SnipsError::InvalidMarker {
        file: file_path.to_path_buf(),
        line: idx + 1,
        content: line.to_string(),
    })?;
    let indent = caps.name("indent").unwrap().as_str().to_string();
    let src_path = caps.name("path").unwrap().as_str().to_string();
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

    Ok(ParsedSnippet {
        indent,
        locator: SnippetLocator {
            path: src_path,
            name: snippet_name,
        },
        old_content: old_content_lines.join("\n"),
    })
}

/// Result of injecting the latest snippet content back into markdown.
struct InjectionResult {
    /// Final rendered markdown text.
    rendered: String,
    /// All snippet references encountered during rendering.
    snippets: Vec<SnippetReport>,
}
