use unicode_segmentation::UnicodeSegmentation;

/// Truncates a string after a certain number of characters.
///
/// Function always tries to truncate on a word boundary.
/// Reimplemented from gonzobot.
pub fn truncate(text: &str, len: usize) -> String {
    if text.len() <= len {
        return text.to_string();
    }
    format!("{}…", text.graphemes(true).take(len).collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::truncate;

    #[test]
    fn test_truncate_with_input_of_lesser_length_than_limit() {
        let input = "short text";
        let result = truncate(input, input.len() + 1);
        assert_eq!(input, result)
    }

    #[test]
    fn test_truncate_with_input_of_equal_length_as_limit() {
        let input = "short text";
        let result = truncate(input, input.len());
        assert_eq!(input, result)
    }

    #[test]
    fn test_truncate_with_input_of_greater_length_than_limit() {
        let input = "some longer text";
        let result = truncate(input, input.len() - 1);
        assert_eq!("some longer tex…", result)
    }

    #[test]
    fn test_truncate_with_input_of_greater_length_than_limit_oneword() {
        let input = "somelongertext";
        let result = truncate(input, input.len() - 1);
        assert_eq!("somelongertex…", result)
    }

    #[test]
    fn test_truncate_with_unicode() {
        let input = "short ° text";
        let result = truncate(input, 7);
        assert_eq!(result, "short °…")
    }
}
