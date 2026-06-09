pub(crate) fn truncate_with_summary(s: &str, max_chars: usize) -> String {
    let extra_chars = s.chars().count().saturating_sub(max_chars);
    if extra_chars == 0 {
        return s.to_string();
    }
    s.chars().take(max_chars).collect::<String>() + &format!("... [{extra_chars} more characters]")
}

pub(crate) fn join_or_default<T, F>(items: &[T], default: &str, extract: F) -> String
where
    F: Fn(&T) -> String,
{
    if items.is_empty() {
        default.to_string()
    } else {
        items.iter().map(extract).collect::<Vec<_>>().join("\n")
    }
}
