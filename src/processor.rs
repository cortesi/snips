use std::{
    fs,
    io::{self, ErrorKind, IsTerminal, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    error::SnipsError,
    snippet::{SNIPPET_ID_CHARS, SnippetRef},
};

/// Policy for handling command-based snippet markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPolicy {
    /// Prompt for confirmation before running each command.
    Prompt,
    /// Allow command execution without prompting.
    Allow,
    /// Disallow command execution entirely.
    Deny,
}

/// A difference between existing markdown content and the current snippet content.
#[derive(Debug)]
pub struct SnippetDiff {
    /// Snippet marker source.
    pub locator: SnippetLocator,
    /// Content currently present in the markdown file.
    pub old_content: String,
    /// Fresh content produced by the snippet source.
    pub new_content: String,
}

/// A snippet reference captured from a markdown file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnippetLocator {
    /// File-based snippet reference.
    File {
        /// Snippet source path relative to the markdown file.
        path: PathBuf,
        /// Optional snippet name inside the source file.
        name: Option<String>,
    },
    /// Command-based snippet reference.
    Command {
        /// Command to execute.
        command: String,
    },
}

impl SnippetLocator {
    /// Render the locator in marker form (e.g., `path/to/file#name` or `!command`).
    pub fn marker(&self) -> String {
        match self {
            Self::File { path, name } => {
                let path = path.to_string_lossy();
                match name {
                    Some(name) => format!("{path}#{name}"),
                    None => path.into_owned(),
                }
            }
            Self::Command { command } => format!("!{command}"),
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

/// Parsed representation of a snippet marker and its content block.
struct ParsedSnippet {
    /// Whitespace indentation preceding the marker.
    marker_indent: String,
    /// Source information recovered from the marker line.
    locator: SnippetLocator,
    /// The block associated with this marker.
    block: SnippetBlock,
}

/// Header metadata used for header-scoped replacements.
struct HeaderLine {
    /// Leading whitespace before the header marker.
    indent: String,
    /// Header level derived from the number of `#` characters.
    level: usize,
    /// Full header line as read from the file.
    raw: String,
}

/// Types of content blocks that can be replaced.
enum SnippetBlock {
    /// Traditional fenced code block replacement.
    CodeFence {
        /// Width of the surrounding code fence in backticks.
        fence_len: usize,
        /// Optional language hint captured from the fence.
        fence_lang: Option<String>,
        /// Original snippet text found between fences.
        old_content: String,
    },
    /// Header-scoped replacement block.
    Header {
        /// Header line for this block.
        header: HeaderLine,
        /// Blank lines between the marker and the header.
        leading_blank_lines: Vec<String>,
        /// Original content under the header.
        old_content: String,
    },
}

/// Cursor for iterating markdown lines with lookahead.
struct LineCursor<'a> {
    /// All lines in the source document.
    lines: Vec<&'a str>,
    /// Current line index within `lines`.
    index: usize,
}

impl<'a> LineCursor<'a> {
    /// Create a new cursor over the provided content.
    fn new(content: &'a str) -> Self {
        Self {
            lines: content.lines().collect(),
            index: 0,
        }
    }

    /// Return the next line and advance the cursor.
    fn next(&mut self) -> Option<(usize, &'a str)> {
        if self.index >= self.lines.len() {
            return None;
        }
        let idx = self.index;
        self.index += 1;
        Some((idx, self.lines[idx]))
    }

    /// Preview the next line without advancing the cursor.
    fn peek(&self) -> Option<(usize, &'a str)> {
        if self.index >= self.lines.len() {
            return None;
        }
        Some((self.index, self.lines[self.index]))
    }

    /// Find the next non-empty line without advancing the cursor.
    fn peek_non_empty(&self) -> Option<(usize, &'a str)> {
        for (idx, line) in self.lines.iter().enumerate().skip(self.index) {
            if !line.trim().is_empty() {
                return Some((idx, *line));
            }
        }
        None
    }

    /// Collect lines up to `end` (exclusive) and advance the cursor.
    fn take_until(&mut self, end: usize) -> Vec<String> {
        let collected = self.lines[self.index..end]
            .iter()
            .map(|line| (*line).to_string())
            .collect();
        self.index = end;
        collected
    }
}

/// Snapshot of a resolved snippet source.
struct ResolvedSnippet {
    /// Resolved snippet text.
    content: String,
    /// Optional language hint for fenced snippets.
    language: Option<String>,
}

/// Regex that validates snippet identifiers.
static SNIPPET_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!(r"^{SNIPPET_ID_CHARS}+$")).expect("snippet name regex"));

/// Apply indentation to every non-blank line in `content`.
fn apply_indentation(content: String, indent: &str) -> String {
    if indent.is_empty() {
        return content;
    }

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

/// Process a single markdown file and optionally write updates in place.
pub fn sync_snippets_in_file(
    path: &Path,
    write: bool,
    command_policy: CommandPolicy,
) -> Result<Option<String>, SnipsError> {
    Ok(sync_snippets_in_file_with_summary(path, write, command_policy)?.rendered)
}

/// Process a single markdown file, returning snippet metadata alongside changes.
pub fn sync_snippets_in_file_with_summary(
    path: &Path,
    write: bool,
    command_policy: CommandPolicy,
) -> Result<RenderSummary, SnipsError> {
    let content = fs::read_to_string(path).map_err(|source| match source.kind() {
        ErrorKind::NotFound => SnipsError::FileNotFound {
            file: path.to_path_buf(),
            source,
        },
        _ => SnipsError::FileReadFailed {
            file: path.to_path_buf(),
            source,
        },
    })?;
    let base = path.parent().unwrap_or(Path::new("."));
    let injection = inject_snippet_content(&content, base, path, command_policy)?;
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
pub fn diff_file(
    path: &Path,
    command_policy: CommandPolicy,
) -> Result<Vec<SnippetDiff>, SnipsError> {
    let content = fs::read_to_string(path).map_err(|source| match source.kind() {
        ErrorKind::NotFound => SnipsError::FileNotFound {
            file: path.to_path_buf(),
            source,
        },
        _ => SnipsError::FileReadFailed {
            file: path.to_path_buf(),
            source,
        },
    })?;
    let base = path.parent().unwrap_or(Path::new("."));
    compute_diffs(&content, base, path, command_policy)
}

/// Scan markdown content for snippet markers and compute diffs against source files.
fn compute_diffs(
    content: &str,
    base: &Path,
    file_path: &Path,
    command_policy: CommandPolicy,
) -> Result<Vec<SnippetDiff>, SnipsError> {
    let mut diffs = Vec::new();
    let mut cursor = LineCursor::new(content);

    while let Some((idx, line)) = cursor.next() {
        if line.trim_start().starts_with("<!-- snips:") {
            let parsed = parse_snippet_block(file_path, idx, line, &mut cursor)?;
            let resolved = resolve_snippet(&parsed.locator, base, command_policy)?;
            let new_content = match &parsed.block {
                SnippetBlock::CodeFence { .. } => {
                    apply_indentation(resolved.content, &parsed.marker_indent)
                }
                SnippetBlock::Header { header, .. } => {
                    apply_indentation(resolved.content, &header.indent)
                }
            };
            let old_content = parsed.old_content();

            if old_content.trim() != new_content.trim() {
                diffs.push(SnippetDiff {
                    locator: parsed.locator.clone(),
                    old_content: old_content.to_string(),
                    new_content,
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
    command_policy: CommandPolicy,
) -> Result<InjectionResult, SnipsError> {
    let mut out = Vec::new();
    let mut snippets = Vec::new();
    let mut cursor = LineCursor::new(content);

    while let Some((idx, line)) = cursor.next() {
        if line.trim_start().starts_with("<!-- snips:") {
            let parsed = parse_snippet_block(file_path, idx, line, &mut cursor)?;
            let resolved = resolve_snippet(&parsed.locator, base, command_policy)?;
            let marker = format!(
                "{}<!-- snips: {} -->",
                parsed.marker_indent,
                parsed.locator.marker()
            );
            out.push(marker);

            match parsed.block {
                SnippetBlock::CodeFence {
                    fence_len,
                    fence_lang,
                    old_content,
                } => {
                    let fence = "`".repeat(fence_len.max(3));
                    let lang_hint = resolved.language.or(fence_lang).unwrap_or_default();
                    if lang_hint.is_empty() {
                        out.push(format!("{}{}", parsed.marker_indent, fence));
                    } else {
                        out.push(format!("{}{}{}", parsed.marker_indent, fence, lang_hint));
                    }

                    let rendered_snippet =
                        apply_indentation(resolved.content, &parsed.marker_indent);
                    let updated = old_content.trim() != rendered_snippet.trim();
                    snippets.push(SnippetReport {
                        locator: parsed.locator.clone(),
                        updated,
                    });
                    out.push(rendered_snippet);
                    out.push(format!("{}{}", parsed.marker_indent, fence));
                }
                SnippetBlock::Header {
                    header,
                    leading_blank_lines,
                    old_content,
                } => {
                    out.extend(leading_blank_lines);
                    out.push(header.raw);
                    out.push(String::new());

                    let rendered_snippet = apply_indentation(resolved.content, &header.indent);
                    let updated = old_content.trim() != rendered_snippet.trim();
                    snippets.push(SnippetReport {
                        locator: parsed.locator.clone(),
                        updated,
                    });
                    if !rendered_snippet.is_empty() {
                        out.push(rendered_snippet);
                    }
                }
            }
        } else {
            out.push(line.to_string());
        }
    }

    Ok(InjectionResult {
        rendered: out.join("\n") + if content.ends_with('\n') { "\n" } else { "" },
        snippets,
    })
}

/// Consume a marker line and its associated block, returning parsed details.
fn parse_snippet_block(
    file_path: &Path,
    idx: usize,
    line: &str,
    cursor: &mut LineCursor<'_>,
) -> Result<ParsedSnippet, SnipsError> {
    let marker = parse_marker_line(file_path, idx, line)?;
    let marker_line = idx + 1;

    let (next_idx, next_line) = cursor
        .peek_non_empty()
        .ok_or(SnipsError::MissingCodeFence(marker_line))?;

    if let Some(header) = parse_header_line(next_line) {
        let leading_blank_lines = cursor.take_until(next_idx);
        cursor.next();
        let old_content = collect_header_body(cursor, header.level);
        return Ok(ParsedSnippet {
            marker_indent: marker.indent,
            locator: marker.locator,
            block: SnippetBlock::Header {
                header,
                leading_blank_lines,
                old_content,
            },
        });
    }

    if next_idx != cursor.index {
        return Err(SnipsError::MissingCodeFence(marker_line));
    }

    let (fence_idx, fence_line) = cursor
        .next()
        .ok_or(SnipsError::MissingCodeFence(marker_line))?;
    let trimmed = fence_line.trim_start();
    if !trimmed.starts_with("```") {
        return Err(SnipsError::MissingCodeFence(marker_line));
    }
    let tick_count = trimmed.chars().take_while(|&c| c == '`').count();
    let closing = "`".repeat(tick_count);
    let fence_lang = trimmed[tick_count..].trim();
    let fence_lang = if fence_lang.is_empty() {
        None
    } else {
        Some(fence_lang.to_string())
    };

    let mut old_content_lines = Vec::new();
    while let Some((_, inner)) = cursor.next() {
        if inner.trim() == closing {
            return Ok(ParsedSnippet {
                marker_indent: marker.indent,
                locator: marker.locator,
                block: SnippetBlock::CodeFence {
                    fence_len: tick_count,
                    fence_lang,
                    old_content: old_content_lines.join("\n"),
                },
            });
        }
        old_content_lines.push(inner.to_string());
    }

    Err(SnipsError::UnterminatedCodeFence {
        file: file_path.to_path_buf(),
        start_line: fence_idx + 1,
    })
}

/// Parse a marker line into its indentation and source.
fn parse_marker_line(file_path: &Path, idx: usize, line: &str) -> Result<MarkerLine, SnipsError> {
    let trimmed_start = line.trim_start();
    let indent_len = line.len() - trimmed_start.len();
    let indent = line[..indent_len].to_string();
    let trimmed = trimmed_start.trim_end();

    let inner = trimmed
        .strip_prefix("<!--")
        .and_then(|value| value.strip_suffix("-->"))
        .map(str::trim)
        .ok_or_else(|| SnipsError::InvalidMarker {
            file: file_path.to_path_buf(),
            line: idx + 1,
            content: line.to_string(),
        })?;

    let payload = inner
        .strip_prefix("snips:")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| SnipsError::InvalidMarker {
            file: file_path.to_path_buf(),
            line: idx + 1,
            content: line.to_string(),
        })?;

    if let Some(command) = payload.strip_prefix('!') {
        let command = command.trim();
        if command.is_empty() {
            return Err(SnipsError::InvalidMarker {
                file: file_path.to_path_buf(),
                line: idx + 1,
                content: line.to_string(),
            });
        }

        return Ok(MarkerLine {
            indent,
            locator: SnippetLocator::Command {
                command: command.to_string(),
            },
        });
    }

    if payload.split_whitespace().count() != 1 {
        return Err(SnipsError::InvalidMarker {
            file: file_path.to_path_buf(),
            line: idx + 1,
            content: line.to_string(),
        });
    }

    let (path_str, name) = if let Some((path, name)) = payload.split_once('#') {
        if name.is_empty() {
            return Err(SnipsError::InvalidMarker {
                file: file_path.to_path_buf(),
                line: idx + 1,
                content: line.to_string(),
            });
        }
        (path, Some(name))
    } else {
        (payload, None)
    };

    if path_str.is_empty() {
        return Err(SnipsError::InvalidMarker {
            file: file_path.to_path_buf(),
            line: idx + 1,
            content: line.to_string(),
        });
    }

    let name = name.map(|value| value.to_string());
    if let Some(name) = &name
        && !SNIPPET_NAME_RE.is_match(name)
    {
        return Err(SnipsError::InvalidMarker {
            file: file_path.to_path_buf(),
            line: idx + 1,
            content: line.to_string(),
        });
    }

    Ok(MarkerLine {
        indent,
        locator: SnippetLocator::File {
            path: PathBuf::from(path_str),
            name,
        },
    })
}

/// Parse an ATX header line, returning its metadata.
fn parse_header_line(line: &str) -> Option<HeaderLine> {
    let indent_len = line.chars().take_while(|c| *c == ' ').count();
    if indent_len > 3 {
        return None;
    }
    let trimmed = &line[indent_len..];
    let hash_count = trimmed.chars().take_while(|c| *c == '#').count();
    if hash_count == 0 || hash_count > 6 {
        return None;
    }
    let rest = &trimmed[hash_count..];
    if !(rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t')) {
        return None;
    }

    Some(HeaderLine {
        indent: line[..indent_len].to_string(),
        level: hash_count,
        raw: line.to_string(),
    })
}

/// Collect the block content under a header until the next header boundary.
fn collect_header_body(cursor: &mut LineCursor<'_>, header_level: usize) -> String {
    let mut body = Vec::new();
    while let Some((_, line)) = cursor.peek() {
        if let Some(next_header) = parse_header_line(line)
            && next_header.level <= header_level
        {
            break;
        }
        cursor.next();
        body.push(line.to_string());
    }
    body.join("\n")
}

/// Resolve a snippet source to its content and optional language hint.
fn resolve_snippet(
    locator: &SnippetLocator,
    base: &Path,
    command_policy: CommandPolicy,
) -> Result<ResolvedSnippet, SnipsError> {
    match locator {
        SnippetLocator::File { path, name } => {
            let snippet = SnippetRef {
                path: base.join(path),
                name: name.clone(),
            };
            let (content, language) = snippet.resolve()?;
            Ok(ResolvedSnippet { content, language })
        }
        SnippetLocator::Command { command } => {
            let content = run_command(command, base, command_policy)?;
            Ok(ResolvedSnippet {
                content,
                language: None,
            })
        }
    }
}

/// Strip leading and trailing blank lines from `content`.
fn strip_outer_blank_lines(content: &str) -> String {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut start = 0;
    let mut end = lines.len();

    while start < end && lines[start].trim().is_empty() {
        start += 1;
    }

    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }

    if start >= end {
        return String::new();
    }

    lines[start..end].join("\n")
}

/// Run a command and return its stdout.
fn run_command(
    command: &str,
    working_dir: &Path,
    command_policy: CommandPolicy,
) -> Result<String, SnipsError> {
    match command_policy {
        CommandPolicy::Allow => {}
        CommandPolicy::Deny => {
            return Err(SnipsError::CommandExecutionDisabled {
                command: command.to_string(),
            });
        }
        CommandPolicy::Prompt => {
            if !confirm_command(command)? {
                return Err(SnipsError::CommandExecutionDenied {
                    command: command.to_string(),
                });
            }
        }
    }

    let (program, args) = split_command(command);

    let output = Command::new(program)
        .args(args)
        .current_dir(working_dir)
        .stdin(Stdio::null())
        .output()
        .map_err(|source| SnipsError::CommandSpawnFailed {
            command: command.to_string(),
            source,
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stderr = if stderr.is_empty() {
            "<empty>".to_string()
        } else {
            stderr
        };
        return Err(SnipsError::CommandFailed {
            command: command.to_string(),
            status: output.status.to_string(),
            stderr,
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(strip_outer_blank_lines(&stdout))
}

/// Confirm command execution when running in prompt mode.
fn confirm_command(command: &str) -> Result<bool, SnipsError> {
    let stdin = io::stdin();
    if !stdin.is_terminal() {
        return Err(SnipsError::CommandConfirmationUnavailable {
            command: command.to_string(),
        });
    }

    let mut stderr = io::stderr();
    writeln!(stderr, "snips wants to run command:\n  {command}")?;
    write!(stderr, "Allow? [y/N]: ")?;
    stderr.flush()?;

    let mut response = String::new();
    stdin.read_line(&mut response)?;
    let response = response.trim().to_ascii_lowercase();
    Ok(matches!(response.as_str(), "y" | "yes"))
}

/// Split a command string into program and arguments.
fn split_command(command: &str) -> (String, Vec<String>) {
    let mut parts = command.split_whitespace();
    let program = parts.next().unwrap_or_default().to_string();
    let args = parts.map(str::to_string).collect();
    (program, args)
}

/// Result of injecting the latest snippet content back into markdown.
struct InjectionResult {
    /// Final rendered markdown text.
    rendered: String,
    /// All snippet references encountered during rendering.
    snippets: Vec<SnippetReport>,
}

impl ParsedSnippet {
    /// Return the original content captured under this marker.
    fn old_content(&self) -> &str {
        match &self.block {
            SnippetBlock::CodeFence { old_content, .. } => old_content,
            SnippetBlock::Header { old_content, .. } => old_content,
        }
    }
}

/// Parsed marker indentation and locator data.
struct MarkerLine {
    /// Indentation captured from the marker line.
    indent: String,
    /// Parsed snippet locator.
    locator: SnippetLocator,
}

#[cfg(test)]
mod tests {
    use super::strip_outer_blank_lines;

    #[test]
    fn strip_outer_blank_lines_removes_surrounding_whitespace() {
        let input = "\n\nLine A\n\nLine B\n\n";
        let expected = "Line A\n\nLine B";
        assert_eq!(strip_outer_blank_lines(input), expected);
    }

    #[test]
    fn strip_outer_blank_lines_handles_all_blank_content() {
        let input = " \n\t\n";
        assert!(strip_outer_blank_lines(input).is_empty());
    }

    #[test]
    fn strip_outer_blank_lines_preserves_non_blank_content() {
        let input = "Line A\nLine B";
        assert_eq!(strip_outer_blank_lines(input), input);
    }
}
