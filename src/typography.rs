const NO_BREAK_SPACE: char = '\u{00A0}';
const SHORT_TAIL: usize = 22;

pub fn tie(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let starts_word = i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '\'');
        let single_letter = matches!(c, 'I' | 'a' | 'A') && starts_word;

        out.push(c);

        if single_letter && chars.get(i + 1) == Some(&' ') {
            out.push(NO_BREAK_SPACE);
            i += 1;
        }

        i += 1;
    }

    out
}

pub fn bind_end(text: &str) -> String {
    let body = text.trim_end();
    let tail = &text[body.len()..];
    let spaces: Vec<usize> = body.match_indices(' ').map(|(i, _)| i).collect();

    let bound = match spaces.as_slice() {
        [.., first, _] if body.len() - first <= SHORT_TAIL => &spaces[spaces.len() - 2..],
        [.., _] => &spaces[spaces.len() - 1..],
        [] => return text.to_string(),
    };

    let mut out = String::with_capacity(text.len());
    let mut from = 0;

    for &space in bound.iter().filter(|&&space| space > 0) {
        out.push_str(&body[from..space]);
        out.push(NO_BREAK_SPACE);
        from = space + 1;
    }

    out.push_str(&body[from..]);
    out.push_str(tail);
    out
}

pub fn bind_paragraph_ends(html: &str) -> String {
    let mut out = String::with_capacity(html.len());

    for (i, part) in html.split("</p>").enumerate() {
        if i > 0 {
            out.push_str("</p>");
        }

        match part.rfind("<p") {
            Some(start) => {
                let (before, paragraph) = part.split_at(start);
                out.push_str(before);
                out.push_str(&bind_last_space(paragraph));
            }
            None => out.push_str(part),
        }
    }

    out
}

fn bind_last_space(paragraph: &str) -> String {
    let mut in_tag = false;
    let mut last_space = None;
    let mut text_after = false;

    for (i, c) in paragraph.char_indices() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            ' ' if !in_tag => {
                last_space = Some(i);
                text_after = false;
            }
            c if !in_tag && !c.is_whitespace() => text_after = true,
            _ => {}
        }
    }

    match last_space {
        Some(i) if text_after && paragraph[..i].chars().any(|c| !c.is_whitespace()) => {
            format!("{}{NO_BREAK_SPACE}{}", &paragraph[..i], &paragraph[i + 1..])
        }
        _ => paragraph.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_single_letter_words_with_the_next_word() {
        assert_eq!(tie("and I build a tool"), "and I\u{a0}build a\u{a0}tool");
    }

    #[test]
    fn leaves_other_words_alone() {
        assert_eq!(tie("I've seen a data cat"), "I've seen a\u{a0}data cat");
        assert_eq!(tie("Rust is fast"), "Rust is fast");
    }

    #[test]
    fn binds_the_end_of_a_text() {
        assert_eq!(
            bind_end("find me as meetzli."),
            "find me\u{a0}as\u{a0}meetzli."
        );
        assert_eq!(
            bind_end("one extraordinarily-long-word two"),
            "one extraordinarily-long-word\u{a0}two"
        );
        assert_eq!(
            bind_end("is built with Dioxus too."),
            "is built with\u{a0}Dioxus\u{a0}too."
        );
        assert_eq!(bind_end("single"), "single");
    }

    #[test]
    fn binds_the_end_of_each_paragraph_outside_tags() {
        let html = "<p>one two <a href=\"x y\">three four</a>.</p>\n<p>five six</p>";

        assert_eq!(
            bind_paragraph_ends(html),
            "<p>one two <a href=\"x y\">three\u{a0}four</a>.</p>\n<p>five\u{a0}six</p>"
        );
    }
}
