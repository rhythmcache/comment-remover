pub fn collapse_whitespace(input: &str, max_newlines: usize) -> String {
    if max_newlines == usize::MAX {
        return input.to_string();
    }

    let mut result = String::with_capacity(input.len());
    let mut empty_count = 0;

    for line in input.split_inclusive('\n') {
        if line.trim().is_empty() {
            empty_count += 1;
        } else {
            if empty_count > 0 {
                let keep = if empty_count <= max_newlines {
                    empty_count
                } else {
                    max_newlines
                };
                for _ in 0..keep {
                    result.push('\n');
                }
                empty_count = 0;
            }
            result.push_str(line);
        }
    }

    if empty_count > 0 {
        let keep = if empty_count <= max_newlines {
            empty_count
        } else {
            max_newlines
        };
        for _ in 0..keep {
            result.push('\n');
        }
    } else if input.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }

    result
}
