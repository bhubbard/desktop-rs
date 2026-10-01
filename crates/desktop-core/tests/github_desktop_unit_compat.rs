use desktop_core::{
    commit::format_commit_message,
    diff::{parse_diff, DiffLineType},
    models::{Author, FileStatusType},
};

/// Ported directly from GitHub Desktop:
/// app/test/unit/diff-parser-test.ts -> 'parses changed files'
#[test]
fn test_github_desktop_diff_parser_changed_files() {
    let diff_text = r#"diff --git a/app/src/lib/diff-parser.ts b/app/src/lib/diff-parser.ts
index e1d4871..3bd3ee0 100644
--- a/app/src/lib/diff-parser.ts
+++ b/app/src/lib/diff-parser.ts
@@ -18,6 +18,7 @@ export function parseRawDiff(lines: ReadonlyArray<string>): Diff {
 
     let numberOfUnifiedDiffLines = 0
 
+
     while (prefixFound) {
 
       // trim any preceding text
@@ -71,12 +72,9 @@ export function parseRawDiff(lines: ReadonlyArray<string>): Diff {
         diffSections.push(new DiffSection(range, diffLines, startDiffSection, endDiffSection))
       } else {
         const diffBody = diffTextBuffer
-
         let startDiffSection: number = 0
         let endDiffSection: number = 0
-
         const diffLines = diffBody.split('\\n')
-
         if (diffSections.length === 0) {
           startDiffSection = 0
           endDiffSection = diffLines.length
@@ -84,10 +82,8 @@ export function parseRawDiff(lines: ReadonlyArray<string>): Diff {
           startDiffSection = numberOfUnifiedDiffLines
           endDiffSection = startDiffSection + diffLines.length
         }
-
         diffSections.push(new DiffSection(range, diffLines, startDiffSection, endDiffSection))
       }
     }
-
     return new Diff(diffSections)
 }
"#;

    let diffs = parse_diff(diff_text);
    assert_eq!(diffs.len(), 1);
    let diff = &diffs[0];
    for (idx, h) in diff.hunks.iter().enumerate() {
        println!("Parsed Hunk {}: {}", idx, h.header);
    }
    assert_eq!(
        diff.old_path.as_deref(),
        Some("app/src/lib/diff-parser.ts")
    );
    assert_eq!(
        diff.new_path.as_deref(),
        Some("app/src/lib/diff-parser.ts")
    );
    assert_eq!(diff.hunks.len(), 3);

    // Hunk 0
    let hunk0 = &diff.hunks[0];
    assert_eq!(hunk0.old_start, 18);
    assert_eq!(hunk0.old_lines, 6);
    assert_eq!(hunk0.new_start, 18);
    assert_eq!(hunk0.new_lines, 7);
    assert_eq!(hunk0.added_count(), 1);
    assert_eq!(hunk0.deleted_count(), 0);

    // Hunk 1
    let hunk1 = &diff.hunks[1];
    assert_eq!(hunk1.old_start, 71);
    assert_eq!(hunk1.old_lines, 12);
    assert_eq!(hunk1.new_start, 72);
    assert_eq!(hunk1.new_lines, 9);
    assert_eq!(hunk1.deleted_count(), 3);

    // Hunk 2
    let hunk2 = &diff.hunks[2];
    assert_eq!(hunk2.old_start, 84);
    assert_eq!(hunk2.old_lines, 10);
    assert_eq!(hunk2.new_start, 82);
    assert_eq!(hunk2.new_lines, 8);
    assert_eq!(hunk2.deleted_count(), 2);
}

/// Ported directly from GitHub Desktop:
/// app/test/unit/diff-parser-test.ts -> 'parses new files'
#[test]
fn test_github_desktop_diff_parser_new_files() {
    let diff_text = r#"diff --git a/testste b/testste
new file mode 100644
index 0000000..f13588b
--- /dev/null
+++ b/testste
@@ -0,0 +1 @@
+asdfasdf
"#;

    let diffs = parse_diff(diff_text);
    assert_eq!(diffs.len(), 1);
    let diff = &diffs[0];
    assert_eq!(diff.old_path, None);
    assert_eq!(diff.new_path.as_deref(), Some("testste"));
    assert_eq!(diff.hunks.len(), 1);

    let hunk = &diff.hunks[0];
    assert_eq!(hunk.old_start, 0);
    assert_eq!(hunk.old_lines, 0);
    assert_eq!(hunk.new_start, 1);
    assert_eq!(hunk.new_lines, 1);
    assert_eq!(hunk.lines.len(), 1);
    assert_eq!(hunk.lines[0].line_type, DiffLineType::Addition);
    assert_eq!(hunk.lines[0].content, "asdfasdf");
    assert_eq!(hunk.lines[0].new_lineno, Some(1));
}

/// Ported directly from GitHub Desktop:
/// app/test/unit/diff-parser-test.ts -> 'parses files containing @@'
#[test]
fn test_github_desktop_diff_parser_files_containing_at_at() {
    let diff_text = r#"diff --git a/test.txt b/test.txt
index 24219cc..bf711a5 100644
--- a/test.txt
+++ b/test.txt
@@ -1 +1 @@
-foo @@
+@@ foo
"#;

    let diffs = parse_diff(diff_text);
    assert_eq!(diffs.len(), 1);
    let diff = &diffs[0];
    assert_eq!(diff.hunks.len(), 1);

    let hunk = &diff.hunks[0];
    assert_eq!(hunk.lines.len(), 2);
    assert_eq!(hunk.lines[0].line_type, DiffLineType::Deletion);
    assert_eq!(hunk.lines[0].content, "foo @@");
    assert_eq!(hunk.lines[1].line_type, DiffLineType::Addition);
    assert_eq!(hunk.lines[1].content, "@@ foo");
}

/// Ported directly from GitHub Desktop:
/// app/test/unit/diff-parser-test.ts -> 'parses new files without a newline at end of file'
#[test]
fn test_github_desktop_diff_parser_no_newline_at_end() {
    let diff_text = r#"diff --git a/test2.txt b/test2.txt
new file mode 100644
index 0000000..faf7da1
--- /dev/null
+++ b/test2.txt
@@ -0,0 +1 @@
+asdasdasd
\ No newline at end of file
"#;

    let diffs = parse_diff(diff_text);
    assert_eq!(diffs.len(), 1);
    let diff = &diffs[0];
    assert_eq!(diff.hunks.len(), 1);

    let hunk = &diff.hunks[0];
    assert_eq!(hunk.lines.len(), 2);
    assert_eq!(hunk.lines[0].line_type, DiffLineType::Addition);
    assert_eq!(hunk.lines[0].content, "asdasdasd");
    assert_eq!(hunk.lines[1].line_type, DiffLineType::NoNewline);
}

/// Ported directly from GitHub Desktop:
/// app/test/unit/status-parser-test.ts
#[test]
fn test_github_desktop_status_parser_porcelain_codes() {
    // Tests mapping of porcelain codes to status indicators
    assert_eq!(FileStatusType::from_porcelain_code('M'), FileStatusType::Modified);
    assert_eq!(FileStatusType::from_porcelain_code('A'), FileStatusType::Added);
    assert_eq!(FileStatusType::from_porcelain_code('D'), FileStatusType::Deleted);
    assert_eq!(FileStatusType::from_porcelain_code('R'), FileStatusType::Renamed);
    assert_eq!(FileStatusType::from_porcelain_code('?'), FileStatusType::Untracked);
    assert_eq!(FileStatusType::from_porcelain_code('.'), FileStatusType::Unmodified);
    assert_eq!(FileStatusType::from_porcelain_code('U'), FileStatusType::Conflicted);
}

/// Ported directly from GitHub Desktop:
/// app/test/unit/format-commit-message-test.ts
#[test]
fn test_github_desktop_format_commit_message() {
    // 'always adds trailing newline'
    assert_eq!(format_commit_message("test", None, &[]), "test\n");
    assert_eq!(format_commit_message("test", Some("test"), &[]), "test\n\ntest\n");

    // 'omits description when null / empty'
    assert_eq!(format_commit_message("test", None, &[]), "test\n");
    assert_eq!(format_commit_message("test", Some(""), &[]), "test\n");

    // 'adds two newlines between summary and description'
    assert_eq!(format_commit_message("foo", Some("bar"), &[]), "foo\n\nbar\n");

    // 'appends trailers to a summary-only message'
    let trailers = vec![
        Author::new("Markus Olsson", "niik@github.com"),
        Author::new("nerdneha", "nerdneha@github.com"),
    ];
    let formatted = format_commit_message("foo", None, &trailers);
    assert_eq!(
        formatted,
        "foo\n\nCo-authored-by: Markus Olsson <niik@github.com>\nCo-authored-by: nerdneha <nerdneha@github.com>\n"
    );

    // 'appends trailers to a summary and description message'
    let formatted_with_body = format_commit_message("foo", Some("bar"), &trailers);
    assert_eq!(
        formatted_with_body,
        "foo\n\nbar\n\nCo-authored-by: Markus Olsson <niik@github.com>\nCo-authored-by: nerdneha <nerdneha@github.com>\n"
    );
}

/// Ported directly from GitHub Desktop:
/// app/test/unit/unique-coauthors-as-authors-test.ts
#[test]
fn test_github_desktop_co_author_parsing() {
    let trailer_line = "Co-authored-by: Markus Olsson <niik@github.com>";
    let author = Author::parse_trailer(trailer_line).expect("should parse");
    assert_eq!(author.name, "Markus Olsson");
    assert_eq!(author.email, "niik@github.com");
    assert_eq!(author.to_trailer(), trailer_line);
}
