//! Standardized candidate profile PDF (C-2), typeset in the FSP brand font JetBrains Mono.
//! The font is monospaced, so line wrapping is exact by character count.

use std::sync::LazyLock;

use krilla::{
    Document,
    color::rgb,
    geom::{PathBuilder, Point, Rect},
    num::NormalizedF32,
    page::PageSettings,
    paint::Fill,
    surface::Surface,
    text::{Font, TextDirection},
};

static REGULAR: LazyLock<Option<Font>> = LazyLock::new(|| {
    Font::new(
        include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf")
            .to_vec()
            .into(),
        0,
    )
});
static BOLD: LazyLock<Option<Font>> = LazyLock::new(|| {
    Font::new(
        include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf")
            .to_vec()
            .into(),
        0,
    )
});

const PAGE_W: f32 = 595.0;
const PAGE_H: f32 = 842.0;
const MARGIN: f32 = 48.0;
const ADVANCE_EM: f32 = 0.6;
const PRIMARY: (u8, u8, u8) = (0x40, 0x2F, 0xFF);
const INK: (u8, u8, u8) = (0x1B, 0x1C, 0x21);
const MUTED: (u8, u8, u8) = (0x5A, 0x5B, 0x66);

#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("embedded font failed to load")]
    Font,
    #[error("pdf: {0:?}")]
    Krilla(krilla::error::KrillaError),
}

pub struct ProfileDoc {
    pub title: String,
    pub subtitle: String,
    pub badge: Option<String>,
    pub sections: Vec<Section>,
    pub footer: String,
}

pub struct Section {
    pub heading: String,
    pub lines: Vec<Line>,
}

pub enum Line {
    Text(String),
    KeyValue(String, String),
    Bullet(String),
}

fn wrap(text: &str, size: f32, width: f32) -> Vec<String> {
    let max = ((width / (size * ADVANCE_EM)).floor() as usize).max(8);
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut current = String::new();
        let mut len = 0usize;
        for word in paragraph.split_whitespace() {
            let wlen = word.chars().count();
            if len > 0 && len + 1 + wlen > max {
                lines.push(std::mem::take(&mut current));
                len = 0;
            }
            if wlen > max {
                let chars: Vec<char> = word.chars().collect();
                for chunk in chars.chunks(max) {
                    if len > 0 {
                        lines.push(std::mem::take(&mut current));
                    }
                    current.extend(chunk);
                    len = chunk.len();
                }
                continue;
            }
            if len > 0 {
                current.push(' ');
                len += 1;
            }
            current.push_str(word);
            len += wlen;
        }
        lines.push(current);
    }
    lines
}

struct Writer<'a> {
    doc: &'a mut Document,
    regular: Font,
    bold: Font,
    pages: Vec<Vec<Op>>,
    y: f32,
}

enum Op {
    Text {
        x: f32,
        y: f32,
        size: f32,
        bold: bool,
        color: (u8, u8, u8),
        text: String,
    },
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: (u8, u8, u8),
    },
}

impl Writer<'_> {
    fn ensure(&mut self, height: f32) {
        if self.y + height > PAGE_H - MARGIN {
            self.pages.push(Vec::new());
            self.y = MARGIN;
        }
    }

    fn op(&mut self, op: Op) {
        if let Some(page) = self.pages.last_mut() {
            page.push(op);
        }
    }

    fn text(&mut self, x: f32, size: f32, bold: bool, color: (u8, u8, u8), text: &str, width: f32) {
        for line in wrap(text, size, width) {
            self.ensure(size * 1.45);
            self.y += size * 1.45;
            let y = self.y;
            self.op(Op::Text {
                x,
                y,
                size,
                bold,
                color,
                text: line,
            });
        }
    }

    fn flush(self) -> Result<(), PdfError> {
        let settings = PageSettings::from_wh(PAGE_W, PAGE_H).ok_or(PdfError::Font)?;
        for ops in self.pages {
            let mut page = self.doc.start_page_with(settings.clone());
            let mut surface = page.surface();
            for op in ops {
                draw(&mut surface, &self.regular, &self.bold, op);
            }
            surface.finish();
            page.finish();
        }
        Ok(())
    }
}

fn fill(color: (u8, u8, u8)) -> Option<Fill> {
    Some(Fill {
        paint: rgb::Color::new(color.0, color.1, color.2).into(),
        opacity: NormalizedF32::ONE,
        rule: Default::default(),
    })
}

fn draw(surface: &mut Surface<'_>, regular: &Font, bold: &Font, op: Op) {
    match op {
        Op::Text {
            x,
            y,
            size,
            bold: is_bold,
            color,
            text,
        } => {
            surface.set_fill(fill(color));
            let font = if is_bold {
                bold.clone()
            } else {
                regular.clone()
            };
            surface.draw_text(
                Point::from_xy(x, y),
                font,
                size,
                &text,
                false,
                TextDirection::Auto,
            );
        }
        Op::Rect { x, y, w, h, color } => {
            if let Some(rect) = Rect::from_xywh(x, y, w, h) {
                let mut pb = PathBuilder::new();
                pb.push_rect(rect);
                if let Some(path) = pb.finish() {
                    surface.set_fill(fill(color));
                    surface.draw_path(&path);
                }
            }
        }
    }
}

pub fn render(profile: &ProfileDoc) -> Result<Vec<u8>, PdfError> {
    let regular = REGULAR.clone().ok_or(PdfError::Font)?;
    let bold = BOLD.clone().ok_or(PdfError::Font)?;
    let mut doc = Document::new();
    let content_w = PAGE_W - 2.0 * MARGIN;
    let mut w = Writer {
        doc: &mut doc,
        regular,
        bold,
        pages: vec![Vec::new()],
        y: MARGIN,
    };

    w.op(Op::Rect {
        x: 0.0,
        y: 0.0,
        w: PAGE_W,
        h: 6.0,
        color: PRIMARY,
    });
    w.text(
        MARGIN,
        9.0,
        true,
        PRIMARY,
        "ФСП · ПРОФИЛЬ КАНДИДАТА",
        content_w,
    );
    w.y += 6.0;
    w.text(MARGIN, 20.0, true, INK, &profile.title, content_w);
    w.text(MARGIN, 11.0, false, MUTED, &profile.subtitle, content_w);
    if let Some(badge) = &profile.badge {
        w.y += 4.0;
        w.text(MARGIN, 11.0, true, PRIMARY, badge, content_w);
    }

    for section in &profile.sections {
        w.ensure(40.0);
        w.y += 16.0;
        w.text(MARGIN, 12.0, true, INK, &section.heading, content_w);
        let y = w.y + 4.0;
        w.op(Op::Rect {
            x: MARGIN,
            y,
            w: content_w,
            h: 0.8,
            color: PRIMARY,
        });
        w.y += 6.0;
        for line in &section.lines {
            match line {
                Line::Text(t) => w.text(MARGIN, 10.0, false, INK, t, content_w),
                Line::Bullet(t) => {
                    w.text(
                        MARGIN + 12.0,
                        10.0,
                        false,
                        INK,
                        &format!("· {t}"),
                        content_w - 12.0,
                    );
                }
                Line::KeyValue(k, v) => {
                    let key_w = 150.0;
                    let start = w.y;
                    w.text(MARGIN, 10.0, false, MUTED, k, key_w);
                    let after_key = w.y;
                    w.y = start;
                    w.text(MARGIN + key_w, 10.0, false, INK, v, content_w - key_w);
                    w.y = w.y.max(after_key);
                }
            }
        }
    }

    w.y += 20.0;
    w.text(MARGIN, 8.0, false, MUTED, &profile.footer, content_w);
    w.flush()?;
    doc.finish().map_err(PdfError::Krilla)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_by_width() {
        let lines = wrap("один два три четыре пять шесть семь", 10.0, 60.0);
        assert!(lines.iter().all(|l| l.chars().count() <= 10));
        assert_eq!(lines.join(" "), "один два три четыре пять шесть семь");
    }

    #[test]
    fn renders_cyrillic_multipage() {
        let doc = ProfileDoc {
            title: "Иван Петров".into(),
            subtitle: "Бэкенд-разработка · Middle".into(),
            badge: Some("Категория подтверждена тестом".into()),
            sections: vec![Section {
                heading: "Опыт".into(),
                lines: (0..120)
                    .map(|i| Line::KeyValue(format!("Пункт {i}"), "Значение ".repeat(12)))
                    .collect(),
            }],
            footer: "Сформировано платформой".into(),
        };
        let pdf = render(&doc).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 1000);
    }
}
