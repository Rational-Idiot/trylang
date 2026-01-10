fn take_while(condition: impl Fn(char) -> bool, s: &str) -> (&str, &str) {
    let count = s
        .char_indices()
        .find_map(|(idx, c)| if condition(c) { None } else { Some(idx) })
        .unwrap_or(s.len());

    (&s[count..], &s[..count])
}

fn take_while_not_empty(
    condition: impl Fn(char) -> bool,
    s: &str,
    err_msg: String,
) -> Result<(&str, &str), String> {
    let (rem, ex) = take_while(condition, s);
    if ex.is_empty() {
        return Err(err_msg);
    } else {
        return Ok((rem, ex));
    }
}

const WHITESPACE: &[char] = &[' ', '\n'];

pub(crate) fn extract_whitespace(s: &str) -> (&str, &str) {
    take_while(|c| WHITESPACE.contains(&c), s)
}

pub(crate) fn extract_whitespace_atleast_one(s: &str) -> Result<(&str, &str), String> {
    take_while_not_empty(
        |c| WHITESPACE.contains(&c),
        s,
        "expected a space".to_string(),
    )
}

pub(crate) fn extract_identifier<'a>(s: &'a str) -> Result<(&'a str, &'a str), String> {
    let num_start = s
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false);

    if num_start {
        Err("expected identifier".into())
    } else {
        Ok(take_while(|c| c.is_ascii_alphanumeric(), s))
    }
}

pub(crate) fn extract_digits(s: &str) -> Result<(&str, &str), String> {
    take_while_not_empty(|c| c.is_ascii_digit(), s, "expected digits".into())
}

pub(crate) fn extract_op(s: &str) -> (&str, &str) {
    match &s[0..1] {
        "+" | "-" | "*" | "/" => {}
        _ => panic!("Unrecognised operator"),
    }

    (&s[1..], &s[0..1])
}

pub(crate) fn tag<'a, 'b>(start_text: &'a str, s: &'b str) -> Result<&'b str, String> {
    if s.starts_with(start_text) {
        Ok(&s[start_text.len()..])
    } else {
        Err(format!("Expected '{}'", start_text))
    }
}

pub(crate) fn sequence<T>(
    parser: impl Fn(&str) -> Result<(&str, T), String>,
    mut s: &str,
) -> Result<(&str, Vec<T>), String> {
    let mut items = Vec::new();

    while let Ok((new_s, item)) = parser(s) {
        s = new_s;
        items.push(item);

        let (new_s, _) = extract_whitespace(s);
        s = new_s;
    }

    Ok((s, items))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_one() {
        assert_eq!(extract_digits("1+2"), Ok(("+2", "1")))
    }

    #[test]
    fn extract_multiple() {
        assert_eq!(extract_digits("10-20"), Ok(("-20", "10")))
    }

    #[test]
    fn do_not_extract_digits_from_invalid_input() {
        assert_eq!(extract_digits("abcd"), Err("expected digits".into()));
    }

    #[test]
    fn extract_digits_with_no_remainder() {
        assert_eq!(extract_digits("100"), Ok(("", "100")));
    }

    #[test]
    fn extract_plus() {
        assert_eq!(extract_op("+2"), ("2", "+"));
    }

    #[test]
    fn extract_minus() {
        assert_eq!(extract_op("-10"), ("10", "-"));
    }

    #[test]
    fn extract_star() {
        assert_eq!(extract_op("*3"), ("3", "*"));
    }

    #[test]
    fn extract_slash() {
        assert_eq!(extract_op("/4"), ("4", "/"));
    }

    #[test]
    fn extract_ws() {
        assert_eq!(extract_whitespace(" 2"), ("2", " "));
    }

    #[test]
    fn extract_mul_ws() {
        assert_eq!(extract_whitespace("       69"), ("69", "       "));
    }

    #[test]
    fn fail_on_num_start() {
        assert_eq!(
            extract_identifier("123abc"),
            Err("expected identifier".to_string())
        );
    }

    #[test]
    fn tag_word() {
        assert_eq!(tag("let", "let a"), Ok(" a"));
    }

    #[test]
    fn extract_alphabetic_ident() {
        assert_eq!(extract_identifier("abcdEFG stop"), Ok((" stop", "abcdEFG")));
    }

    #[test]
    fn extract_alphanumeric_ident() {
        assert_eq!(extract_identifier("foobar1()"), Ok(("()", "foobar1")));
    }

    #[test]
    fn extract_newlines_or_spaces() {
        assert_eq!(extract_whitespace(" \n   \n\nabc"), ("abc", " \n   \n\n"));
    }

    #[test]
    fn do_not_extract_spaces1_when_input_does_not_start_with_them() {
        assert_eq!(
            extract_whitespace_atleast_one("blah"),
            Err("expected a space".to_string()),
        );
    }
}
