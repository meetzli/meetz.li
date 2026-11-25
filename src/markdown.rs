use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, html};

use crate::highlight::highlight;
use crate::typography::{bind_paragraph_ends, tie};

pub fn render(markdown: &str) -> String {
    let mut out = String::new();
    html::push_html(&mut out, highlighted(markdown));
    bind_paragraph_ends(&out)
}

fn highlighted(markdown: &str) -> impl Iterator<Item = Event<'_>> {
    let mut lang = String::new();
    let mut code = String::new();
    let mut in_code = false;

    Parser::new_ext(markdown, Options::empty()).filter_map(move |event| match event {
        Event::Start(Tag::CodeBlock(kind)) => {
            in_code = true;
            code.clear();
            lang = match kind {
                CodeBlockKind::Fenced(lang) => lang.to_string(),
                CodeBlockKind::Indented => String::new(),
            };
            None
        }
        Event::Text(text) if in_code => {
            code.push_str(&text);
            None
        }
        Event::End(TagEnd::CodeBlock) => {
            in_code = false;
            let body = highlight(&code, &lang);

            Some(Event::Html(CowStr::from(format!(
                "<div class=\"codeblock\"><pre><code class=\"language-{lang}\">{body}</code></pre></div>\n"
            ))))
        }
        Event::Text(text) => Some(Event::Text(CowStr::from(tie(&text)))),
        other => Some(other),
    })
}
