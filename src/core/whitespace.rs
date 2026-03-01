pub fn collapse_whitespace(input: &str, max_newlines: usize) -> String {
    if max_newlines == usize::MAX {
        return input.to_string();
    }

    let lines: Vec<&str> = input.split_inclusive('\n').collect();
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    let n = lines.len();

    while i < n {
        if lines[i].trim().is_empty() {
            let start = i;
            while i < n && lines[i].trim().is_empty() {
                i += 1;
            }
            let end = i;
            let run_len = end - start;

            let at_start = start == 0;
            let at_end = end == n;

            let keep = if at_start || at_end {
                if run_len <= max_newlines + 1 {
                    run_len
                } else {
                    max_newlines + 1
                }
            } else {
                if run_len <= max_newlines {
                    run_len
                } else {
                    max_newlines
                }
            };

            for j in start..start + keep {
                result.push_str(lines[j]);
            }
        } else {
            result.push_str(lines[i]);
            i += 1;
        }
    }

    if input.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }

    result
}
