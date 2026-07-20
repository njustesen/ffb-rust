
/// Replace `$1`, `$2`, ... placeholders in `template` with the given parameters.
///
/// 1:1 translation of Java `StringTool.bind(String, Object[])`: returns an
/// empty string when the template is empty or no parameters are given, and
/// silently drops placeholders whose index exceeds the parameter count.
pub fn bind(template: &str, params: &[&str]) -> String {
    let mut result = String::new();
    if is_provided(Some(template)) && !params.is_empty() {
        let mut chars = template.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek().is_some_and(|d| d.is_ascii_digit()) {
                // Collect consecutive digits (Java pattern: [$]([0-9]+))
                let mut digits = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        digits.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Ok(idx) = digits.parse::<usize>() {
                    if idx >= 1 && idx <= params.len() {
                        result.push_str(params[idx - 1]);
                    }
                    // Java: index >= parameters.length appends nothing
                    // (unmatched placeholders are dropped).
                }
            } else {
                result.push(ch);
            }
        }
    }
    result
}

/// Format a number with thousands separators: 2_130_000 → "2,130,000".
pub fn format_thousands(n: i64) -> String {
    let s = n.to_string();
    let (sign, digits) = if s.starts_with('-') { ("-", &s[1..]) } else { ("", s.as_str()) };
    let len = digits.len();
    let mut result = String::new();
    for (i, ch) in digits.chars().enumerate() {
        let remaining = len - i;
        if i > 0 && remaining % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    format!("{}{}", sign, result)
}

/// Join items with ", " and "and" before the last: ["a","b","c"] → "a, b and c".
pub fn build_enumeration(items: &[&str]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].to_owned(),
        _ => {
            let mut out = String::new();
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    if i == items.len() - 1 {
                        out.push_str(" and ");
                    } else {
                        out.push_str(", ");
                    }
                }
                out.push_str(item);
            }
            out
        }
    }
}

pub fn is_number(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// Java: `StringTool.isProvided(Object)` — true if non-null and non-empty (`toString().length() > 0`).
pub fn is_provided(s: Option<&str>) -> bool {
    matches!(s, Some(v) if !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_basic() {
        assert_eq!(bind("Hello $1, you are $2!", &["World", "great"]), "Hello World, you are great!");
    }

    #[test]
    fn bind_unmatched_placeholder_dropped() {
        // Java StringTool.bind drops placeholders with no matching parameter.
        assert_eq!(bind("$1 and $3", &["a"]), "a and ");
    }

    #[test]
    fn format_thousands_basic() {
        assert_eq!(format_thousands(2_130_000), "2,130,000");
        assert_eq!(format_thousands(1000), "1,000");
        assert_eq!(format_thousands(42), "42");
    }

    #[test]
    fn enumeration() {
        assert_eq!(build_enumeration(&["a"]), "a");
        assert_eq!(build_enumeration(&["a", "b"]), "a and b");
        assert_eq!(build_enumeration(&["a", "b", "c"]), "a, b and c");
    }

    #[test]
    fn is_number_checks() {
        assert!(is_number("42"));
        assert!(!is_number("4x"));
        assert!(!is_number(""));
    }

    #[test]
    fn is_provided_checks() {
        assert!(is_provided(Some("hi")));
        assert!(!is_provided(Some("")));
        assert!(!is_provided(None));
    }
}
