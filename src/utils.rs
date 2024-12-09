use rand::Rng;

pub fn generate_random_string(length: usize) -> String {
    let charset = b"abcdef0123456789";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

/// Used to find all values in a comma/newline seperated list, ignoring comments
pub fn get_list_string(value: &str) -> Vec<String> {
    let mut values: Vec<String> = Vec::new();

    for line in value.lines() {
        if line.starts_with('#') {
            continue;
        }

        let line_without_comment = line
            .split('#')
            .take(1)
            .collect::<String>()
            .trim()
            .to_string();

        if line_without_comment.is_empty() {
            continue;
        }

        line_without_comment
            .split(",")
            .for_each(|s| values.push(s.trim().to_string()));
    }

    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_list_string_new_line_with_comments() {
        let mut text = String::new();
        text.push_str("# this is a comment\n");
        text.push_str("123456\n");
        text.push_str("\n");
        text.push_str("   \n");
        text.push_str("\n");
        text.push_str("\t\n");
        text.push_str("654321\n");
        text.push_str("    888  \n");
        text.push_str("    999 # another comment \n");
        text.push_str("   \n");

        assert_eq!(
            get_list_string(&text),
            vec![
                "123456".to_string(),
                "654321".to_string(),
                "888".to_string(),
                "999".to_string(),
            ]
        );
    }

    #[test]
    fn get_list_string_basic() {
        let mut text = String::new();
        text.push_str("");
        assert_eq!(get_list_string(&text), Vec::<String>::new());
    }

    #[test]
    fn get_list_string_comma_seperated() {
        let mut text = String::new();
        text.push_str("123456,987654, 456789");
        assert_eq!(get_list_string(&text), vec!["123456", "987654", "456789"]);
    }
}
