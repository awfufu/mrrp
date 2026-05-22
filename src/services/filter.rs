pub fn filter_rule_lines(body: &str) -> String {
    let mut filtered = body
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect::<Vec<_>>()
        .join("\n");

    if !filtered.is_empty() {
        filtered.push('\n');
    }

    filtered
}
