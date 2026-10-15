const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "pure", "guard", "throw",
];

const PRIMITIVES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "str",
];

fn escape(text: &str, out: &mut String) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

fn span(class: &str, text: &str, out: &mut String) {
    out.push_str("<span class=\"");
    out.push_str(class);
    out.push_str("\">");
    escape(text, out);
    out.push_str("</span>");
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

pub fn highlight(code: &str, lang: &str) -> String {
    let mut out = String::with_capacity(code.len() * 2);

    if lang != "rust" {
        escape(code, &mut out);
        return out;
    }

    let chars: Vec<char> = code.chars().collect();
    let text = |from: usize, to: usize| chars[from..to].iter().collect::<String>();
    let mut i = 0;
    let mut after_fn = false;

    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();

        if ch == '/' && next == Some('/') {
            let end = (i..chars.len())
                .find(|&j| chars[j] == '\n')
                .unwrap_or(chars.len());
            span("hl-c", &text(i, end), &mut out);
            i = end;
            continue;
        }

        if ch == '/' && next == Some('*') {
            let end = (i + 2..chars.len().saturating_sub(1))
                .find(|&j| chars[j] == '*' && chars[j + 1] == '/')
                .map(|j| j + 2)
                .unwrap_or(chars.len());
            span("hl-c", &text(i, end), &mut out);
            i = end;
            continue;
        }

        if ch == '#'
            && matches!(next, Some('[') | Some('!'))
            && let Some(end) = (i..chars.len()).find(|&j| chars[j] == ']').map(|j| j + 1)
        {
            span("hl-a", &text(i, end), &mut out);
            i = end;
            continue;
        }

        if ch == '"' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '"' {
                j += if chars[j] == '\\' { 2 } else { 1 };
            }
            let end = (j + 1).min(chars.len());
            span("hl-s", &text(i, end), &mut out);
            i = end;
            continue;
        }

        if ch == '\'' {
            let is_char = next == Some('\\') || chars.get(i + 2) == Some(&'\'');

            if is_char {
                let mut j = i + 1;
                while j < chars.len() && chars[j] != '\'' {
                    j += if chars[j] == '\\' { 2 } else { 1 };
                }
                let end = (j + 1).min(chars.len());
                span("hl-s", &text(i, end), &mut out);
                i = end;
                continue;
            }

            if next.is_some_and(is_ident_start) {
                let end = (i + 1..chars.len())
                    .find(|&j| !is_ident(chars[j]))
                    .unwrap_or(chars.len());
                span("hl-l", &text(i, end), &mut out);
                i = end;
                continue;
            }
        }

        if ch.is_ascii_digit() {
            let mut j = i;
            while j < chars.len()
                && (is_ident(chars[j])
                    || (chars[j] == '.' && chars.get(j + 1).is_some_and(|c| c.is_ascii_digit())))
            {
                j += 1;
            }
            span("hl-n", &text(i, j), &mut out);
            i = j;
            continue;
        }

        if ch == '<' && next == Some('-') {
            let end = if chars.get(i + 2) == Some(&'?') {
                i + 3
            } else {
                i + 2
            };
            span("hl-k", &text(i, end), &mut out);
            i = end;
            continue;
        }

        if is_ident_start(ch) {
            let end = (i..chars.len())
                .find(|&j| !is_ident(chars[j]))
                .unwrap_or(chars.len());
            let word = text(i, end);
            let is_macro = chars.get(end) == Some(&'!') && chars.get(end + 1) != Some(&'=');
            let is_call = chars.get(end) == Some(&'(');

            let class = if KEYWORDS.contains(&word.as_str()) {
                Some("hl-k")
            } else if is_macro {
                Some("hl-m")
            } else if after_fn || is_call {
                Some("hl-f")
            } else if PRIMITIVES.contains(&word.as_str())
                || word.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            {
                Some("hl-t")
            } else {
                None
            };

            match class {
                Some(class) => span(class, &word, &mut out),
                None => escape(&word, &mut out),
            }

            after_fn = word == "fn";
            i = end;
            continue;
        }

        if !ch.is_whitespace() {
            after_fn = false;
        }

        escape(&ch.to_string(), &mut out);
        i += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_the_common_tokens() {
        let html = highlight("let x = Some(10); // hi\nhdo! { x <- a; }", "rust");

        assert!(html.contains("<span class=\"hl-k\">let</span>"));
        assert!(html.contains("<span class=\"hl-n\">10</span>"));
        assert!(html.contains("<span class=\"hl-c\">// hi</span>"));
        assert!(html.contains("<span class=\"hl-m\">hdo</span>!"));
        assert!(html.contains("<span class=\"hl-k\">&lt;-</span>"));
    }

    #[test]
    fn escapes_other_languages() {
        assert_eq!(highlight("a < b", "text"), "a &lt; b");
    }

    #[test]
    fn tells_lifetimes_from_chars() {
        let html = highlight("fn f<'a>(c: char) { 'x'; }", "rust");

        assert!(html.contains("<span class=\"hl-l\">'a</span>"));
        assert!(html.contains("<span class=\"hl-s\">'x'</span>"));
    }
}
