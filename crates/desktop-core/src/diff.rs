use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
    Header,
    NoNewline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffHunk {
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    pub fn added_count(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Addition)
            .count()
    }

    pub fn deleted_count(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Deletion)
            .count()
    }

    /// Generates a standalone, unified patch for this individual hunk
    /// suitable for applying with `git apply --cached` or `git apply --reverse`.
    pub fn to_patch(&self, file_path: &str) -> String {
        let mut patch = String::new();
        patch.push_str(&format!("--- a/{file_path}\n"));
        patch.push_str(&format!("+++ b/{file_path}\n"));
        patch.push_str(&self.header);
        if !self.header.ends_with('\n') {
            patch.push('\n');
        }

        for line in &self.lines {
            let prefix = match line.line_type {
                DiffLineType::Context => " ",
                DiffLineType::Addition => "+",
                DiffLineType::Deletion => "-",
                DiffLineType::NoNewline => "\\",
                DiffLineType::Header => "@",
            };
            patch.push_str(prefix);
            patch.push_str(&line.content);
            if !line.content.ends_with('\n') && line.line_type != DiffLineType::NoNewline {
                patch.push('\n');
            }
        }
        patch
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diff {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub is_binary: bool,
    pub hunks: Vec<DiffHunk>,
}

impl Diff {
    pub fn file_path(&self) -> &str {
        self.new_path
            .as_deref()
            .or(self.old_path.as_deref())
            .unwrap_or("")
    }

    pub fn added_count(&self) -> usize {
        self.hunks.iter().map(|h| h.added_count()).sum()
    }

    pub fn deleted_count(&self) -> usize {
        self.hunks.iter().map(|h| h.deleted_count()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.hunks.is_empty() && !self.is_binary
    }
}

/// Parses the output of `git diff` or `git diff --cached` into structured `Diff` models.
pub fn parse_diff(raw_diff: &str) -> Vec<Diff> {
    let mut diffs = Vec::new();
    let lines: Vec<&str> = raw_diff.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.starts_with("diff --git ") {
            let mut old_path = None;
            let mut new_path = None;
            let mut is_binary = false;
            let mut hunks = Vec::new();

            // Extract file paths from "diff --git a/path b/path"
            let parts: Vec<&str> = line.split(" b/").collect();
            if parts.len() >= 2 {
                let first_part = parts[0];
                if let Some(a_idx) = first_part.find(" a/") {
                    old_path = Some(first_part[a_idx + 3..].to_string());
                }
                new_path = Some(parts[1..].join(" b/"));
            }

            i += 1;

            // Process headers before hunks
            while i < lines.len()
                && !lines[i].starts_with("diff --git ")
                && !lines[i].starts_with("@@ ")
            {
                let hdr = lines[i];
                if let Some(stripped) = hdr.strip_prefix("--- a/") {
                    old_path = Some(stripped.to_string());
                } else if hdr.starts_with("--- /dev/null") {
                    old_path = None;
                } else if let Some(stripped) = hdr.strip_prefix("+++ b/") {
                    new_path = Some(stripped.to_string());
                } else if hdr.starts_with("+++ /dev/null") {
                    new_path = None;
                } else if hdr.contains("Binary files") || hdr.contains("GIT binary patch") {
                    is_binary = true;
                }
                i += 1;
            }

            // Process hunks
            while i < lines.len() && !lines[i].starts_with("diff --git ") {
                let hunk_line = lines[i];
                if hunk_line.starts_with("@@ ") {
                    if let Some(hunk) = parse_hunk_header(hunk_line) {
                        let mut current_hunk = hunk;
                        let mut old_lineno = current_hunk.old_start;
                        let mut new_lineno = current_hunk.new_start;

                        i += 1;
                        while i < lines.len()
                            && !lines[i].starts_with("@@ ")
                            && !lines[i].starts_with("diff --git ")
                        {
                            let content_line = lines[i];
                            if let Some(first_char) = content_line.chars().next() {
                                let line_content = if content_line.len() > 1 {
                                    content_line[1..].to_string()
                                } else {
                                    String::new()
                                };

                                match first_char {
                                    ' ' => {
                                        current_hunk.lines.push(DiffLine {
                                            line_type: DiffLineType::Context,
                                            content: line_content,
                                            old_lineno: Some(old_lineno),
                                            new_lineno: Some(new_lineno),
                                        });
                                        old_lineno += 1;
                                        new_lineno += 1;
                                    }
                                    '+' => {
                                        current_hunk.lines.push(DiffLine {
                                            line_type: DiffLineType::Addition,
                                            content: line_content,
                                            old_lineno: None,
                                            new_lineno: Some(new_lineno),
                                        });
                                        new_lineno += 1;
                                    }
                                    '-' => {
                                        current_hunk.lines.push(DiffLine {
                                            line_type: DiffLineType::Deletion,
                                            content: line_content,
                                            old_lineno: Some(old_lineno),
                                            new_lineno: None,
                                        });
                                        old_lineno += 1;
                                    }
                                    '\\' => {
                                        current_hunk.lines.push(DiffLine {
                                            line_type: DiffLineType::NoNewline,
                                            content: line_content,
                                            old_lineno: None,
                                            new_lineno: None,
                                        });
                                    }
                                    _ => {
                                        // Ignore unexpected line
                                    }
                                }
                            }
                            i += 1;
                        }
                        hunks.push(current_hunk);
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }

            diffs.push(Diff {
                old_path,
                new_path,
                is_binary,
                hunks,
            });
        } else {
            i += 1;
        }
    }

    diffs
}

/// Parses "@@ -old_start,old_lines +new_start,new_lines @@ header"
fn parse_hunk_header(header: &str) -> Option<DiffHunk> {
    if !header.starts_with("@@ -") {
        return None;
    }

    let end_idx = header[4..].find(" @@")?;
    let range_str = &header[4..4 + end_idx];
    let parts: Vec<&str> = range_str.split(" +").collect();
    if parts.len() != 2 {
        return None;
    }

    let old_range = parts[0];
    let new_range = parts[1];

    let (old_start, old_lines) = parse_range(old_range)?;
    let (new_start, new_lines) = parse_range(new_range)?;

    Some(DiffHunk {
        header: header.to_string(),
        old_start,
        old_lines,
        new_start,
        new_lines,
        lines: Vec::new(),
    })
}

fn parse_range(s: &str) -> Option<(u32, u32)> {
    if let Some(comma_pos) = s.find(',') {
        let start: u32 = s[..comma_pos].parse().ok()?;
        let lines: u32 = s[comma_pos + 1..].parse().ok()?;
        Some((start, lines))
    } else {
        let start: u32 = s.parse().ok()?;
        Some((start, 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diff_single_file() {
        let raw = r#"diff --git a/src/main.rs b/src/main.rs
index e69de29..49c5e3d 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
-    println!("hello");
+    println!("hello world");
+    println!("goodbye");
 }
"#;
        let diffs = parse_diff(raw);
        assert_eq!(diffs.len(), 1);
        let diff = &diffs[0];
        assert_eq!(diff.file_path(), "src/main.rs");
        assert_eq!(diff.added_count(), 2);
        assert_eq!(diff.deleted_count(), 1);
        assert_eq!(diff.hunks.len(), 1);

        let hunk = &diff.hunks[0];
        assert_eq!(hunk.old_start, 1);
        assert_eq!(hunk.old_lines, 3);
        assert_eq!(hunk.new_start, 1);
        assert_eq!(hunk.new_lines, 4);

        let patch = hunk.to_patch("src/main.rs");
        assert!(patch.contains("--- a/src/main.rs"));
        assert!(patch.contains("+++ b/src/main.rs"));
        assert!(patch.contains("+    println!(\"hello world\");"));
    }

    #[test]
    fn test_parse_binary_diff() {
        let raw = r#"diff --git a/image.png b/image.png
new file mode 100644
index 0000000..f3a12bc
Binary files /dev/null and b/image.png differ
"#;
        let diffs = parse_diff(raw);
        assert_eq!(diffs.len(), 1);
        assert!(diffs[0].is_binary);
        assert_eq!(diffs[0].file_path(), "image.png");
    }
}
