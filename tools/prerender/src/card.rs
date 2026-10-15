use std::sync::Arc;

use crate::Result;
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use resvg::{tiny_skia, usvg};

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 630;

const BG: &str = "#1d2021";
const BAND: &str = "#282828";
const TEXT: &str = "#ebdbb2";
const MUTED: &str = "#a89984";
const LINE: &str = "#3c3836";
const COMMENT: &str = "#928374";
const ACCENT: &str = "#d65d0e";

const LEFT: u32 = 72;
const RIGHT: u32 = 1128;

pub const SERIF: &[u8] = include_bytes!("../fonts/IBMPlexSerif-Regular.ttf");
const SERIF_MEDIUM: &[u8] = include_bytes!("../fonts/IBMPlexSerif-Medium.ttf");
const MONO: &[u8] = include_bytes!("../fonts/IBMPlexMono-Regular.ttf");

pub enum Kind {
    Site { tagline: String },
    List { heading: String, lines: Vec<String> },
    Post { title: String, date: String },
}

pub struct Fonts {
    options: usvg::Options<'static>,
    medium: FontRef<'static>,
}

impl Fonts {
    pub fn load() -> Self {
        let mut db = usvg::fontdb::Database::new();

        for data in [SERIF, SERIF_MEDIUM, MONO] {
            db.load_font_data(data.to_vec());
        }

        let options = usvg::Options {
            fontdb: Arc::new(db),
            ..Default::default()
        };

        Self {
            options,
            medium: FontRef::try_from_slice(SERIF_MEDIUM).expect("IBM Plex Serif Medium"),
        }
    }
}

pub fn render(kind: &Kind, route: &str, fonts: &Fonts) -> Result<Vec<u8>> {
    rasterize(&svg(kind, route, fonts), fonts, WIDTH, HEIGHT)
}

pub fn rasterize(svg: &str, fonts: &Fonts, width: u32, height: u32) -> Result<Vec<u8>> {
    let tree = usvg::Tree::from_str(svg, &fonts.options)?;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or("cannot allocate the image")?;

    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );

    Ok(pixmap.encode_png()?)
}

fn svg(kind: &Kind, route: &str, fonts: &Fonts) -> String {
    let mut body = String::new();

    match kind {
        Kind::Site { tagline } => {
            body += &text(
                LEFT,
                340,
                Style::new("Serif", 500, 150, TEXT, -4),
                "meetzli",
            );
            body += &text(LEFT, 420, Style::new("Serif", 400, 38, MUTED, 0), tagline);
        }

        Kind::List { heading, lines } => {
            body += &text(LEFT, 250, Style::new("Serif", 500, 110, TEXT, -3), heading);

            for (i, line) in lines.iter().take(4).enumerate() {
                body += &text(
                    LEFT,
                    330 + i as u32 * 46,
                    Style::new("Mono", 400, 30, MUTED, 0),
                    &format!("- {line}"),
                );
            }
        }

        Kind::Post { title, date } => {
            let (size, lines) = [88.0, 76.0, 64.0, 54.0]
                .into_iter()
                .map(|size| {
                    (
                        size,
                        wrap(title, &fonts.medium, size, -2.0, (RIGHT - LEFT) as f32),
                    )
                })
                .find(|(_, lines)| lines.len() <= 3)
                .unwrap_or_else(|| {
                    (
                        54.0,
                        wrap(title, &fonts.medium, 54.0, -2.0, (RIGHT - LEFT) as f32),
                    )
                });

            let line_height = size * 1.08;
            let block = 52.0 + lines.len() as f32 * line_height;
            let top = 322.0 - block / 2.0;
            let first = top + 52.0 + size * 0.85;

            body += &text(
                LEFT,
                (top + 26.0) as u32,
                Style::new("Mono", 400, 26, ACCENT, 0),
                date,
            );

            for (i, line) in lines.iter().enumerate() {
                body += &text(
                    LEFT,
                    (first + i as f32 * line_height) as u32,
                    Style::new("Serif", 500, size as u32, TEXT, -2),
                    line,
                );
            }
        }
    }

    let tab = if route == "/" {
        String::new()
    } else {
        route.to_string()
    };

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}">
  <rect width="{WIDTH}" height="{HEIGHT}" fill="{BG}"/>
  {logo}
  {body}
  <rect y="526" width="{WIDTH}" height="104" fill="{BAND}"/>
  <rect y="526" width="{WIDTH}" height="2" fill="{LINE}"/>
  {comment}
</svg>"#,
        logo = text(LEFT, 84, Style::new("Serif", 400, 34, TEXT, 2), "meetzli"),
        comment = text(
            LEFT,
            588,
            Style::new("Mono", 400, 28, COMMENT, 0),
            &format!("// meetz.li{tab}")
        ),
    )
}

#[derive(Clone, Copy)]
struct Style {
    family: &'static str,
    weight: u32,
    size: u32,
    fill: &'static str,
    spacing: i32,
}

impl Style {
    fn new(family: &'static str, weight: u32, size: u32, fill: &'static str, spacing: i32) -> Self {
        Self {
            family,
            weight,
            size,
            fill,
            spacing,
        }
    }
}

fn text(x: u32, y: u32, style: Style, content: &str) -> String {
    let Style {
        family,
        weight,
        size,
        fill,
        spacing,
    } = style;

    format!(
        r#"<text x="{x}" y="{y}" font-family="IBM Plex {family}" font-weight="{weight}" font-size="{size}" letter-spacing="{spacing}" fill="{fill}">{}</text>"#,
        xml(content),
    )
}

fn xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_long_titles_onto_more_lines() {
        let font = FontRef::try_from_slice(SERIF_MEDIUM).unwrap();
        let title = "How the borrow checker turned my macro into a very long afternoon";

        assert!(wrap(title, &font, 88.0, -2.0, 1000.0).len() > 1);
        assert_eq!(wrap("short", &font, 88.0, -2.0, 1000.0), vec!["short"]);
    }

    #[test]
    fn renders_a_png_for_every_kind() {
        let fonts = Fonts::load();
        let kinds = [
            Kind::Site {
                tagline: "x".into(),
            },
            Kind::List {
                heading: "Blog".into(),
                lines: vec!["a & b".into()],
            },
            Kind::Post {
                title: "What hdo! expands to".into(),
                date: "2026-10-14".into(),
            },
        ];

        for kind in &kinds {
            let png = render(kind, "/blog", &fonts).unwrap();

            assert_eq!(&png[1..4], b"PNG");
        }
    }
}
