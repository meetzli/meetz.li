use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use serde::Deserialize;

const SERIF: &[u8] = include_bytes!("../../prerender/fonts/IBMPlexSerif-Regular.ttf");
const MEDIUM: &[u8] = include_bytes!("../../prerender/fonts/IBMPlexSerif-Medium.ttf");
const MONO: &[u8] = include_bytes!("../../prerender/fonts/IBMPlexMono-Regular.ttf");
pub const IMAGE_VERSION: &str = "projects-v1";

#[cfg(target_arch = "wasm32")]
mod edge;

#[derive(Deserialize)]
pub struct Owner {
    pub login: String,
    #[serde(default)]
    pub avatar_url: String,
}

#[derive(Deserialize)]
pub struct Repo {
    pub owner: Owner,
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub stargazers_count: u64,
    pub forks_count: u64,
    pub open_issues_count: u64,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub contributors: u64,
    #[serde(default)]
    pub avatar: String,
}

pub fn route(path: &str) -> Option<(String, bool)> {
    let path = path.strip_prefix("/gh/")?.trim_end_matches('/');
    let (repo, image) = path
        .strip_suffix("/image.png")
        .map_or((path, false), |repo| (repo, true));
    let parts: Vec<_> = repo.split('/').collect();
    (parts.len() == 2
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.len() <= 100
                && *part != "."
                && *part != ".."
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        }))
    .then(|| (repo.into(), image))
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn html(repo: &str, description: &str, origin: &str) -> String {
    let title = escape(repo);
    let description = escape(description);
    let url = format!("{}/gh/{repo}", escape(origin));
    let image = format!("{url}/image.png?v={IMAGE_VERSION}");
    let target = format!("https://github.com/{repo}");
    format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>{title}</title><link rel="canonical" href="{url}"><meta name="description" content="{description}"><meta property="og:type" content="website"><meta property="og:title" content="{title}"><meta property="og:description" content="{description}"><meta property="og:url" content="{url}"><meta property="og:image" content="{image}"><meta property="og:image:width" content="1200"><meta property="og:image:height" content="600"><meta name="twitter:card" content="summary_large_image"><meta name="twitter:title" content="{title}"><meta name="twitter:description" content="{description}"><meta name="twitter:image" content="{image}"><meta http-equiv="refresh" content="0;url={target}"></head><body><a href="{target}">Open {title} on GitHub</a></body></html>"#
    )
}

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 600;

const BG: &str = "#1d2021";
const TEXT: &str = "#ebdbb2";
const MUTED: &str = "#a89984";
const ACCENT: &str = "#d65d0e";

const STAR: &str = "M8 .25a.75.75 0 0 1 .673.418l1.882 3.815 4.21.612a.75.75 0 0 1 .416 1.279l-3.046 2.97.719 4.192a.751.751 0 0 1-1.088.791L8 12.347l-3.766 1.98a.75.75 0 0 1-1.088-.79l.72-4.194L.818 6.374a.75.75 0 0 1 .416-1.28l4.21-.611L7.327.668A.75.75 0 0 1 8 .25Z";
const FORK: &str = "M5 5.372v.878c0 .414.336.75.75.75h4.5a.75.75 0 0 0 .75-.75v-.878a2.25 2.25 0 1 1 1.5 0v.878a2.25 2.25 0 0 1-2.25 2.25h-1.5v2.128a2.251 2.251 0 1 1-1.5 0V8.5h-1.5A2.25 2.25 0 0 1 3.5 6.25v-.878a2.25 2.25 0 1 1 1.5 0ZM5 3.25a.75.75 0 1 0-1.5 0 .75.75 0 0 0 1.5 0Zm6.75.75a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5Zm-3 8.75a.75.75 0 1 0-1.5 0 .75.75 0 0 0 1.5 0Z";
const ISSUE: &str = "M8 9.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3ZM8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Z";
const PEOPLE: &str = "M2 5.5a3.5 3.5 0 1 1 5.898 2.549 5.508 5.508 0 0 1 3.034 4.084.75.75 0 1 1-1.482.235 4 4 0 0 0-7.9 0 .75.75 0 0 1-1.482-.236A5.507 5.507 0 0 1 3.102 8.05 3.493 3.493 0 0 1 2 5.5ZM11 4a3.001 3.001 0 0 1 2.22 5.018 5.01 5.01 0 0 1 2.56 3.012.749.749 0 0 1-.885.954.752.752 0 0 1-.549-.514 3.507 3.507 0 0 0-2.522-2.372.75.75 0 0 1-.574-.73v-.352a.75.75 0 0 1 .416-.672A1.5 1.5 0 0 0 11 5.5.75.75 0 0 1 11 4Zm-5.5-.5a2 2 0 1 0-.001 3.999A2 2 0 0 0 5.5 3.5Z";
const MARK: &str = "M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z";

fn language_color(language: Option<&str>) -> &'static str {
    match language {
        Some("Rust") => "#dea584",
        Some("TypeScript") => "#3178c6",
        Some("JavaScript") => "#f1e05a",
        Some("Python") => "#3572a5",
        _ => ACCENT,
    }
}

fn xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn svg(repo: &Repo) -> String {
    let regular = FontRef::try_from_slice(SERIF).unwrap();
    let lines = wrap(
        repo.description.as_deref().unwrap_or(""),
        &regular,
        34.0,
        0.0,
        700.0,
    );
    let mut description = String::new();

    for (i, line) in lines.iter().take(2).enumerate() {
        description += &format!(
            r#"<text x="80" y="{}" font-family="IBM Plex Serif" font-size="34" fill="{MUTED}">{}</text>"#,
            400 + i * 46,
            xml(line)
        );
    }

    let stats = [
        (
            PEOPLE,
            repo.contributors,
            if repo.contributors == 1 {
                "Contributor"
            } else {
                "Contributors"
            },
        ),
        (ISSUE, repo.open_issues_count, "Issues"),
        (STAR, repo.stargazers_count, "Stars"),
        (FORK, repo.forks_count, "Forks"),
    ];
    let mut row = String::new();

    for (i, (icon, value, label)) in stats.iter().enumerate() {
        let x = 80 + i * 190;

        row += &format!(
            r#"<g transform="translate({x} 478) scale(2)" fill="{MUTED}"><path d="{icon}"/></g>
  <text x="{}" y="506" font-family="IBM Plex Mono" font-size="34" fill="{TEXT}">{value}</text>
  <text x="{x}" y="548" font-family="IBM Plex Mono" font-size="22" fill="{MUTED}">{label}</text>"#,
            x + 44,
        );
    }

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}">
  <defs><clipPath id="avatar"><rect x="900" y="70" width="220" height="220" rx="32"/></clipPath></defs>
  <rect width="{WIDTH}" height="{HEIGHT}" fill="{BG}"/>
  <text x="80" y="200" font-family="IBM Plex Serif" font-size="72" fill="{TEXT}">{owner}/</text>
  <text x="80" y="300" font-family="IBM Plex Serif" font-weight="500" font-size="96" letter-spacing="-2" fill="{TEXT}">{name}</text>
  {description}
  <image href="{avatar}" x="900" y="70" width="220" height="220" clip-path="url(#avatar)"/>
  {row}
  <g transform="translate(1040 470) scale(4.5)" fill="{MUTED}"><path d="{MARK}"/></g>
  <rect y="{bar_y}" width="{WIDTH}" height="30" fill="{color}"/>
</svg>"##,
        owner = xml(&repo.owner.login),
        name = xml(&repo.name),
        avatar = repo.avatar,
        bar_y = HEIGHT - 30,
        color = language_color(repo.language.as_deref()),
    );

    svg
}

pub fn wrap(text: &str, font: &FontRef<'_>, size: f32, spacing: f32, max: f32) -> Vec<String> {
    let scaled = font.as_scaled(PxScale::from(size));
    let width = |s: &str| -> f32 {
        s.chars()
            .map(|c| scaled.h_advance(font.glyph_id(c)) + spacing)
            .sum()
    };

    let mut lines: Vec<String> = Vec::new();

    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if width(&format!("{line} {word}")) <= max => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }

    lines
}

pub fn png(repo: &Repo) -> Result<Vec<u8>, String> {
    let mut options = resvg::usvg::Options::default();
    for font in [SERIF, MEDIUM, MONO] {
        options.fontdb_mut().load_font_data(font.to_vec());
    }
    let tree = resvg::usvg::Tree::from_str(&svg(repo), &options).map_err(|e| e.to_string())?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(1200, 600).ok_or("image allocation failed")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_routes() {
        assert_eq!(
            route("/gh/bytematebot/raskell/image.png"),
            Some(("bytematebot/raskell".into(), true))
        );
        for invalid in [
            "/gh/a",
            "/gh/a/b/extra",
            "/gh/../b",
            "/gh/a/%22",
            "/gh/a/b?x",
        ] {
            assert!(route(invalid).is_none(), "{invalid}");
        }
    }

    #[test]
    fn escapes_metadata_and_renders_png() {
        let repo: Repo = serde_json::from_str(r#"{"owner":{"login":"test"},"name":"repo","description":"<script> & \"test\"","language":null,"stargazers_count":1,"forks_count":2,"open_issues_count":3}"#).unwrap();
        let html = html(
            "test/repo",
            repo.description.as_deref().unwrap(),
            "https://meetz.li",
        );
        assert!(html.contains("&lt;script&gt; &amp; &quot;test&quot;"));
        assert!(html.contains("https://meetz.li/gh/test/repo/image.png"));
        let png = png(&repo).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 1200);
    }
}
