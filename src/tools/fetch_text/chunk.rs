pub fn truncate_with_hint(content: &str, start_index: usize, max_length: usize) -> String {
    let original_len = content.len();
    if start_index >= original_len {
        return "<error>No more content available.</error>".to_string();
    }
    let end = (start_index + max_length).min(original_len);
    let mut slice = content[start_index..end].to_string();
    if end < original_len {
        slice.push_str(&format!(
            "\n\n<error>Content truncated. Call this tool again with start_index={} to get more.</error>",
            end
        ));
    }
    slice
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_with_hint_handles_bounds() {
        let s = "abcdef";
        assert!(truncate_with_hint(s, 10, 3).contains("No more content"));
        assert_eq!(truncate_with_hint(s, 0, 3)[..3], *"abc");
    }
}
