use ra_ap_rustc_lexer::{tokenize, FrontmatterAllowed, TokenKind};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub struct Violation {
    pub file: String,
    pub line: usize,
    pub message: String,
}

struct Comment {
    line: usize,
    text: String,
}

pub fn run() -> Result<(), Vec<Violation>> {
    let root = crate::workspace_root();
    let files = collect_source_files(root);
    let mut violations = Vec::new();
    for file in &files {
        lint_file(file, &mut violations);
    }
    check_unsafe_forbid(&files, &mut violations);
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

pub fn run_cli() -> ExitCode {
    match run() {
        Ok(()) => {
            println!("lint: clean");
            ExitCode::from(0)
        }
        Err(violations) => {
            print_violations(&violations);
            ExitCode::from(1)
        }
    }
}

pub fn print_violations(violations: &[Violation]) {
    for violation in violations {
        eprintln!(
            "{}:{}: {}",
            violation.file, violation.line, violation.message
        );
    }
    eprintln!("lint: {} violation(s)", violations.len());
}

fn collect_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    files
}

fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
    let skip = ["target", ".git", "runs", "private", "tests"];
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if skip.contains(&name.as_str()) {
            continue;
        }
        if path.is_dir() {
            walk(&path, files);
        } else if is_source_file(&path) {
            files.push(path);
        }
    }
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs") | Some("wgsl")
    )
}

fn lint_file(path: &Path, violations: &mut Vec<Violation>) {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(_) => return,
    };
    let display = path.to_string_lossy().into_owned();
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => {
            let ranges = test_module_ranges(&source);
            for comment in rust_comments(&source) {
                if is_license_header(&comment.text) {
                    continue;
                }
                if in_ranges(comment.line, &ranges) {
                    continue;
                }
                violations.push(Violation {
                    file: display.clone(),
                    line: comment.line,
                    message: "comment in production Rust source".to_string(),
                });
            }
        }
        Some("wgsl") => {
            for comment in wgsl_comments(&source) {
                violations.push(Violation {
                    file: display.clone(),
                    line: comment.line,
                    message: "comment in WGSL source".to_string(),
                });
            }
        }
        _ => {}
    }
}

fn rust_comments(source: &str) -> Vec<Comment> {
    let mut comments = Vec::new();
    let mut offset = 0usize;
    for token in tokenize(source, FrontmatterAllowed::No) {
        let len = token.len as usize;
        if matches!(
            token.kind,
            TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
        ) {
            let line = source[..offset].matches('\n').count() + 1;
            comments.push(Comment {
                line,
                text: source[offset..offset + len].to_string(),
            });
        }
        offset += len;
    }
    comments
}

fn wgsl_comments(source: &str) -> Vec<Comment> {
    let bytes = source.as_bytes();
    let mut comments = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            line += 1;
            i += 1;
            continue;
        }
        if bytes[i] == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                let start_line = line;
                let start = i;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                comments.push(Comment {
                    line: start_line,
                    text: source[start..i].to_string(),
                });
                continue;
            }
            if bytes[i + 1] == b'*' {
                let start_line = line;
                let start = i;
                i += 2;
                let mut depth = 1usize;
                while i < bytes.len() && depth > 0 {
                    if bytes[i] == b'\n' {
                        line += 1;
                    }
                    if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                        depth += 1;
                        i += 2;
                    } else if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                comments.push(Comment {
                    line: start_line,
                    text: source[start..i].to_string(),
                });
                continue;
            }
        }
        i += 1;
    }
    comments
}

fn is_license_header(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("copyright") || lower.contains("spdx") || lower.contains("licen")
}

fn test_module_ranges(source: &str) -> Vec<(usize, usize)> {
    let lines: Vec<&str> = source.lines().collect();
    let mut ranges = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        if lines[i].trim() == "#[cfg(test)]" {
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            if j < lines.len()
                && lines[j].trim_start().starts_with("mod ")
                && lines[j].trim_end().ends_with('{')
            {
                let mut depth = 0usize;
                let mut end = lines.len();
                for (k, line) in lines.iter().enumerate().skip(j) {
                    depth += line.chars().filter(|c| *c == '{').count();
                    depth = depth.saturating_sub(line.chars().filter(|c| *c == '}').count());
                    if depth == 0 {
                        end = k;
                        break;
                    }
                }
                ranges.push((i + 1, end + 1));
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    ranges
}

fn in_ranges(line: usize, ranges: &[(usize, usize)]) -> bool {
    ranges
        .iter()
        .any(|(start, end)| line >= *start && line <= *end)
}

fn check_unsafe_forbid(files: &[PathBuf], violations: &mut Vec<Violation>) {
    for file in files {
        let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name != "main.rs" && name != "lib.rs" {
            continue;
        }
        let source = fs::read_to_string(file).unwrap_or_default();
        if !source.contains("#![forbid(unsafe_code)]") {
            violations.push(Violation {
                file: file.to_string_lossy().into_owned(),
                line: 1,
                message: "crate root missing #![forbid(unsafe_code)]".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_inside_string_is_not_a_comment() {
        assert!(rust_comments("let s = \"// not a comment\";").is_empty());
    }

    #[test]
    fn real_comment_is_detected() {
        assert_eq!(rust_comments("// real comment\nfn main() {}").len(), 1);
    }

    #[test]
    fn nested_wgsl_block_comment_is_detected() {
        assert_eq!(wgsl_comments("/* outer /* nested */ still */").len(), 1);
    }

    #[test]
    fn license_header_is_exempt() {
        assert!(is_license_header("// Copyright 2026"));
        assert!(!is_license_header("// an explanatory note"));
    }
}
