/// The complete visible title, including a truncation ellipsis, stays bounded.
pub const CHAT_TITLE_MAX_CHARS: usize = 48;

/// Derives a title from the first non-empty prompt or first usable attachment.
pub fn derive_chat_title<'a>(
    prompt: Option<&str>,
    attachment_file_names: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    let prompt_title = prompt.and_then(compact_nonempty);
    let attachment_title = attachment_file_names.into_iter().find_map(compact_nonempty);
    prompt_title
        .or(attachment_title)
        .map(|candidate| truncate_with_ellipsis(&candidate))
}

fn compact_nonempty(value: &str) -> Option<String> {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!compact.is_empty()).then_some(compact)
}

fn truncate_with_ellipsis(value: &str) -> String {
    let character_count = value.chars().count();
    if character_count <= CHAT_TITLE_MAX_CHARS {
        return value.to_owned();
    }
    let mut bounded = value
        .chars()
        .take(CHAT_TITLE_MAX_CHARS - 1)
        .collect::<String>();
    bounded.push('…');
    bounded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonempty_prompt_wins_and_whitespace_is_compacted() {
        assert_eq!(
            derive_chat_title(Some("  Build\n a   release plan "), ["fallback.md"]),
            Some("Build a release plan".into())
        );
    }

    #[test]
    fn first_usable_attachment_names_an_attachment_only_chat() {
        assert_eq!(
            derive_chat_title(Some(" \n "), [" ", "architecture.png", "later.txt"]),
            Some("architecture.png".into())
        );
    }

    #[test]
    fn truncation_is_unicode_safe_and_includes_the_ellipsis_in_the_bound() {
        let title = derive_chat_title(Some(&"界".repeat(60)), []).unwrap();
        assert_eq!(title.chars().count(), CHAT_TITLE_MAX_CHARS);
        assert!(title.ends_with('…'));
    }

    #[test]
    fn an_empty_submission_has_no_title() {
        assert_eq!(derive_chat_title(Some("  "), []), None);
    }
}
