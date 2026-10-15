use dioxus::prelude::*;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, html};

use crate::data::{POSTS, visible_posts};
use crate::highlight::highlight;
use crate::route::Route;
use crate::typography::{bind_paragraph_ends, tie};

#[component]
pub fn Blog() -> Element {
    if visible_posts().next().is_none() {
        return rsx! {
            article {
                p { "Nothing here yet." }
            }
        };
    }

    rsx! {
        document::Title { "Blog | meetzli" }
        article {
            ul { class: "post-list",
                for post in visible_posts() {
                    li {
                        span { class: "post-date", "{post.date}" }
                        Link {
                            to: Route::BlogPost { slug: post.slug.to_string() },
                            "{post.title}"
                        }
                        if post.draft {
                            span { class: "post-draft", "draft" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn BlogPost(slug: String) -> Element {
    let post = POSTS
        .iter()
        .find(|post| post.slug == slug && (!post.draft || cfg!(debug_assertions)));

    let Some(post) = post else {
        return rsx! {
            article {
                p { "No such post. Back to the " Link { to: Route::Blog {}, "blog" } "." }
            }
        };
    };

    let mut html_out = String::new();
    html::push_html(&mut html_out, highlighted(post.body));
    let html_out = bind_paragraph_ends(&html_out);

    rsx! {
        document::Title { "{post.title} | meetzli" }
        article { class: "post",
            h1 { "{post.title}" }
            p { class: "post-meta", "{post.date}" }
            div { class: "post-body", dangerous_inner_html: html_out }
        }
    }
}

fn highlighted(markdown: &str) -> impl Iterator<Item = Event<'_>> {
    let mut lang = String::new();
    let mut code = String::new();
    let mut in_code = false;

    Parser::new_ext(markdown, Options::empty()).filter_map(move |event| {
        match event {
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
        }
    })
}
