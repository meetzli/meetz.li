mod card;
mod repos;
mod serve;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[path = "../../../src/data.rs"]
#[allow(dead_code)]
mod data;

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use card::Kind;

const SITE: &str = "https://meetz.li";
const ACCENT: u32 = 0xd65d0e;
const START: &str = "<!-- prerender:start -->";
const END: &str = "<!-- prerender:end -->";

struct Page {
    route: String,
    title: String,
    description: String,
    image: String,
    button: &'static str,
    kind: Kind,
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("serve") {
        return serve_command();
    }

    if std::env::args().nth(1).as_deref() == Some("repos") {
        let out = std::env::args()
            .nth(2)
            .unwrap_or_else(|| "assets/repos".into());

        if let Err(error) = repos::run(Path::new(&out)) {
            fail(&error.to_string());
        }

        return;
    }

    let mut public = None;
    let mut drafts = false;
    let mut base = SITE.to_string();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--drafts" => drafts = true,
            "--base" => match args.next() {
                Some(url) => base = url.trim_end_matches('/').to_string(),
                None => fail("--base needs a url"),
            },
            path => public = Some(PathBuf::from(path)),
        }
    }

    let public = public.unwrap_or_else(|| PathBuf::from("target/dx/meetzli/release/web/public"));
    let debug_build = public.components().any(|part| part.as_os_str() == "debug");

    if let Err(error) = run(&public, drafts || debug_build, &base) {
        fail(&error.to_string());
    }
}

fn serve_command() {
    let mut port = 8081;
    let mut upstream = "http://127.0.0.1:8080".to_string();
    let mut args = std::env::args().skip(2);

    while let Some(arg) = args.next() {
        match (arg.as_str(), args.next()) {
            ("--port", Some(value)) => {
                port = value
                    .parse()
                    .unwrap_or_else(|_| fail("--port needs a number"))
            }
            ("--upstream", Some(value)) => upstream = value.trim_end_matches('/').to_string(),
            _ => fail("usage: prerender serve [--port 8081] [--upstream http://127.0.0.1:8080]"),
        }
    }

    if let Err(error) = serve::run(port, &upstream) {
        fail(&error.to_string());
    }
}

fn fail(message: &str) -> ! {
    eprintln!("prerender: {message}");
    std::process::exit(1);
}

fn run(public: &Path, drafts: bool, base: &str) -> Result<()> {
    let fonts = card::Fonts::load();
    fs::create_dir_all(public.join("og"))?;

    for page in pages(drafts) {
        let out = if page.route == "/" {
            public.join("index.html")
        } else {
            public
                .join(page.route.trim_start_matches('/'))
                .join("index.html")
        };
        let source = fs::read_to_string(&out).map_err(|e| {
            format!(
                "cannot read {} (is {} in [build] prerender?): {e}",
                out.display(),
                page.route
            )
        })?;
        let source = strip_title(&strip_previous(&source));
        let png = card::render(&page.kind, &page.route, &fonts)?;
        let html = source.replacen(
            "</head>",
            &format!("  {}\n  </head>", head(&page, base, &version(&png))?),
            1,
        );

        fs::write(&out, html)?;
        fs::write(public.join("og").join(format!("{}.png", page.image)), png)?;

        println!(
            "{:<28} -> {}  +  og/{}.png",
            page.route,
            out.strip_prefix(public).unwrap_or(&out).display(),
            page.image
        );
    }

    Ok(())
}

fn strip_title(html: &str) -> String {
    match (html.find("<title>"), html.find("</title>")) {
        (Some(start), Some(end)) if start < end => {
            let after = html[end + "</title>".len()..].trim_start_matches('\n');
            format!("{}{after}", &html[..start])
        }
        _ => html.to_string(),
    }
}

fn strip_previous(html: &str) -> String {
    match (html.find(START), html.find(END)) {
        (Some(start), Some(end)) if start < end => {
            let before = html[..start].trim_end_matches([' ', '\n']);
            format!("{before}{}", &html[end + END.len()..])
        }
        _ => html.to_string(),
    }
}

fn pages(drafts: bool) -> Vec<Page> {
    let projects: Vec<&str> = data::PROJECTS.iter().map(|p| p.name).collect();
    let posts: Vec<&data::Post> = data::POSTS.iter().filter(|p| drafts || !p.draft).collect();
    let titles: Vec<&str> = posts.iter().map(|p| p.title).collect();

    let mut pages = vec![
        Page {
            route: "/".into(),
            title: data::PERSONAL_INFO.name.into(),
            description: data::PERSONAL_INFO.bio.into(),
            image: "site".into(),
            button: "Visit site",
            kind: Kind::Site {
                tagline: data::PERSONAL_INFO.bio.into(),
            },
        },
        Page {
            route: "/projects".into(),
            title: "Projects | meetzli".into(),
            description: projects.join(", "),
            image: "projects".into(),
            button: "See projects",
            kind: Kind::List {
                heading: "Projects".into(),
                lines: owned(&projects),
            },
        },
        Page {
            route: "/blog".into(),
            title: "Blog | meetzli".into(),
            description: if titles.is_empty() {
                "Writing by meetzli.".into()
            } else {
                titles.join(", ")
            },
            image: "blog".into(),
            button: "Read the blog",
            kind: Kind::List {
                heading: "Blog".into(),
                lines: owned(&titles),
            },
        },
    ];

    for post in posts {
        pages.push(Page {
            route: format!("/blog/{}", post.slug),
            title: format!("{} | meetzli", post.title),
            description: summary(post.body, 240),
            image: format!("post-{}", post.slug),
            button: "Read post",
            kind: Kind::Post {
                title: post.title.into(),
                date: post.date.into(),
            },
        });
    }

    pages
}

fn version(png: &[u8]) -> String {
    let hash = png.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ *byte as u64).wrapping_mul(0x100000001b3)
    });

    format!("{:08x}", hash >> 32)
}

fn owned(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn summary(markdown: &str, max: usize) -> String {
    let first = markdown.trim().split("\n\n").next().unwrap_or("");

    let mut text = String::new();
    let mut rest = first;
    while let Some(open) = rest.find('[') {
        let Some(close) = rest[open..].find("](").map(|i| open + i) else {
            break;
        };
        let Some(end) = rest[close..].find(')').map(|i| close + i) else {
            break;
        };
        text.push_str(&rest[..open]);
        text.push_str(&rest[open + 1..close]);
        rest = &rest[end + 1..];
    }
    text.push_str(rest);

    let text = text.replace(['`', '*', '_'], "");
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

    if text.chars().count() <= max {
        return text;
    }

    let mut sentences = String::new();

    for sentence in text.split_inclusive(". ") {
        if sentences.chars().count() + sentence.chars().count() > max {
            break;
        }

        sentences.push_str(sentence);
    }

    let sentences = sentences.trim_end();

    if sentences.chars().count() >= 40 {
        return sentences.to_string();
    }

    let cut: String = text.chars().take(max).collect();
    let cut = cut.rsplit_once(' ').map_or(cut.as_str(), |(head, _)| head);
    format!("{cut}...")
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

fn embed(page: &Page, url: &str, image: &str) -> Result<String> {
    let heading = page.title.trim_end_matches(" | meetzli");

    let payload = json!({
        "component": {
            "type": 17,
            "accent_color": ACCENT,
            "components": [
                { "type": 12, "items": [{ "media": { "url": image }, "description": heading }] },
                {
                    "type": 9,
                    "components": [{ "type": 10, "content": "\u{200b}" }],
                    "accessory": { "type": 2, "style": 5, "url": url, "label": page.button }
                }
            ]
        }
    });

    let json = payload.to_string().replace('<', "\\u003c");

    if json.len() > 3000 {
        return Err(format!(
            "the embed for {} is {} bytes, the limit is 3000",
            page.route,
            json.len()
        )
        .into());
    }

    Ok(json)
}

fn head(page: &Page, base: &str, version: &str) -> Result<String> {
    let url = if page.route == "/" {
        base.to_string()
    } else {
        format!("{base}{}", page.route)
    };
    let image = format!("{base}/og/{}.png?v={version}", page.image);
    let kind = if matches!(page.kind, Kind::Post { .. }) {
        "article"
    } else {
        "website"
    };
    let title = escape(&page.title);
    let description = escape(&page.description);
    let embed = embed(page, &url, &image)?;

    Ok(format!(
        r#"{START}
    <title>{title}</title>
    <meta name="description" content="{description}" />
    <link rel="canonical" href="{url}" />
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:type" content="{kind}" />
    <meta property="og:url" content="{url}" />
    <meta property="og:image" content="{image}" />
    <meta property="og:image:width" content="1200" />
    <meta property="og:image:height" content="630" />
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content="{title}" />
    <meta name="twitter:description" content="{description}" />
    <meta name="twitter:image" content="{image}" />
    <script id="discord:component-embed" type="application/json">{embed}</script>
    {END}"#
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_changes_with_the_image() {
        assert_ne!(version(b"one"), version(b"two"));
        assert_eq!(version(b"one"), version(b"one"));
    }

    #[test]
    fn summary_drops_markdown() {
        let text = summary(
            "I wrote `hdo!` for [Rust](https://rust-lang.org).\n\nSecond.",
            100,
        );

        assert_eq!(text, "I wrote hdo! for Rust.");
    }

    #[test]
    fn summary_prefers_whole_sentences() {
        let text = "This first sentence is long enough to count. Second one runs on and on and on.";

        assert_eq!(
            summary(text, 60),
            "This first sentence is long enough to count."
        );
    }

    #[test]
    fn summary_cuts_at_a_word() {
        assert_eq!(summary("one two three four", 9), "one two...");
    }

    #[test]
    fn the_title_of_the_page_is_replaced() {
        let page = "<head>\n<title>meetzli</title>\n<meta charset=\"utf-8\">\n</head>";

        assert_eq!(strip_title(page), "<head>\n<meta charset=\"utf-8\">\n</head>");
    }

    #[test]
    fn running_twice_does_not_stack_tags() {
        let once = format!("<head>\n  {START}\n x\n  {END}\n</head>");

        assert_eq!(strip_previous(&once), "<head>\n</head>");
    }

    #[test]
    fn every_embed_fits_in_the_limit() {
        for page in pages(true) {
            let url = format!("{SITE}{}", page.route);

            assert!(
                embed(&page, &url, &format!("{SITE}/og/x.png")).is_ok(),
                "{}",
                page.route
            );
        }
    }
}
