use std::path::Path;

/// Format file size in human readable format
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Check if a file is likely to be a text file based on its extension
pub fn is_text_file(path: &Path) -> bool {
    const BINARY_EXTENSIONS: &[&str] = &[
        "exe", "dll", "so", "dylib", "bin", "obj", "o", "jpg", "jpeg", "png", "gif", "bmp", "ico",
        "mp3", "mp4", "avi", "mov", "wav", "zip", "tar", "gz", "rar", "7z", "pdf", "doc", "docx",
        "xls", "xlsx",
    ];

    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        !BINARY_EXTENSIONS.contains(&ext.as_str())
    } else {
        true // Assume files without extensions might be text
    }
}

/// Extract a snippet from text around the query terms
pub fn extract_snippet(text: &str, query: &str, max_length: usize) -> String {
    let query_words: Vec<&str> = query
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .collect();

    if query_words.is_empty() || text.is_empty() {
        return truncate_text(text, max_length);
    }

    if let Some((pos, word_len)) = find_best_match_position(text, &query_words) {
        extract_snippet_around_position(text, pos, word_len, max_length)
    } else {
        truncate_text(text, max_length)
    }
}

/// Find the best position to extract a snippet from
fn find_best_match_position(text: &str, query_words: &[&str]) -> Option<(usize, usize)> {
    let text_lower = text.to_lowercase();
    let mut best_pos = None;
    let mut best_word_len = 0;

    for word in query_words {
        let word_lower = word.to_lowercase();
        if let Some(pos) = text_lower.find(&word_lower) {
            if best_pos.is_none() || pos < best_pos.unwrap() {
                best_pos = Some(pos);
                best_word_len = word.len();
            }
        }
    }

    best_pos.map(|pos| (pos, best_word_len))
}

/// Extract a snippet around a specific position
fn extract_snippet_around_position(
    text: &str,
    pos: usize,
    word_len: usize,
    max_length: usize,
) -> String {
    let context_size = (max_length - word_len) / 2;
    let start = pos.saturating_sub(context_size);
    let end = (pos + word_len + context_size).min(text.len());

    let mut snippet = text[start..end].to_string();
    snippet = clean_snippet(&snippet);

    let prefix = if start > 0 { "..." } else { "" };
    let suffix = if end < text.len() { "..." } else { "" };

    format!("{}{}{}", prefix, snippet.trim(), suffix)
}

/// Clean up a snippet by removing excessive whitespace
fn clean_snippet(snippet: &str) -> String {
    snippet
        .replace('\n', " ")
        .replace('\t', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Truncate text to a maximum length
fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() > max_length {
        format!("{}...", &text[..max_length].trim())
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_is_text_file() {
        assert!(is_text_file(Path::new("test.rs")));
        assert!(is_text_file(Path::new("README.md")));
        assert!(is_text_file(Path::new("file_without_extension")));
        assert!(!is_text_file(Path::new("image.jpg")));
        assert!(!is_text_file(Path::new("binary.exe")));
    }

    #[test]
    fn test_extract_snippet() {
        let text = "This is a long piece of text that contains the word function somewhere in the middle and we want to extract a snippet around it.";
        let query = "function";
        let snippet = extract_snippet(text, query, 50);

        assert!(snippet.contains("function"));
        assert!(snippet.starts_with("..."));
        assert!(snippet.ends_with("..."));
    }

    #[test]
    fn test_clean_snippet() {
        let input = "This  has\t\ttabs\nand\n\nnewlines    and     spaces";
        let expected = "This has tabs and newlines and spaces";
        assert_eq!(clean_snippet(input), expected);
    }
}
