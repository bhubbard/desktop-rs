use crate::models::Author;

/// Formats a git commit message with summary, optional extended description,
/// and standard GitHub `Co-authored-by: Name <email>` trailers.
pub fn format_commit_message(
    summary: &str,
    description: Option<&str>,
    co_authors: &[Author],
) -> String {
    let mut msg = summary.trim().to_string();

    let desc = description.map(|d| d.trim()).filter(|d| !d.is_empty());
    if let Some(d) = desc {
        msg.push_str("\n\n");
        msg.push_str(d);
    }

    if !co_authors.is_empty() {
        msg.push_str("\n\n");

        for (i, author) in co_authors.iter().enumerate() {
            if i > 0 {
                msg.push('\n');
            }
            msg.push_str(&author.to_trailer());
        }
    }

    msg.push('\n');
    msg
}

/// Splits a commit message into body content and extracted `Co-authored-by:` trailers.
pub fn extract_co_authors(raw_message: &str) -> (String, Vec<Author>) {
    let mut clean_lines = Vec::new();
    let mut authors = Vec::new();

    for line in raw_message.lines() {
        if let Some(author) = Author::parse_trailer(line) {
            authors.push(author);
        } else {
            clean_lines.push(line);
        }
    }

    // Trim trailing empty lines
    while let Some(last) = clean_lines.last() {
        if last.trim().is_empty() {
            clean_lines.pop();
        } else {
            break;
        }
    }

    (clean_lines.join("\n"), authors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_commit_message_simple() {
        let msg = format_commit_message("feat: add something", None, &[]);
        assert_eq!(msg, "feat: add something\n");
    }

    #[test]
    fn test_format_commit_message_with_co_authors() {
        let authors = vec![
            Author::new("Octocat", "octocat@github.com"),
            Author::new("Hubot", "hubot@github.com"),
        ];
        let msg = format_commit_message(
            "feat: collaborative feature",
            Some("Details here"),
            &authors,
        );
        assert!(msg.contains("Co-authored-by: Octocat <octocat@github.com>"));
        assert!(msg.contains("Co-authored-by: Hubot <hubot@github.com>"));

        let (body, parsed_authors) = extract_co_authors(&msg);
        assert_eq!(parsed_authors.len(), 2);
        assert_eq!(parsed_authors[0].name, "Octocat");
        assert_eq!(parsed_authors[1].email, "hubot@github.com");
        assert!(body.contains("feat: collaborative feature\n\nDetails here"));
    }
}
