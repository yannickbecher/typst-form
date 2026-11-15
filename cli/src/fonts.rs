//! Fonts for field text: the PDF standard fonts plus TrueType fonts the
//! document registered with `form-font`. All use WinAnsiEncoding, which every
//! viewer can edit with.

use std::collections::HashMap;

use anyhow::{Context, Result, ensure};
use lopdf::{Dictionary, Document, Object, Stream, dictionary};

/// Font resources (for /DR and appearance streams) and their widths.
pub struct Fonts {
    pub resources: Dictionary,
    /// Advance width per WinAnsi code in 1/1000 em, per resource name.
    widths: HashMap<String, [u16; 256]>,
}

impl Fonts {
    /// Add the standard fonts and the given TrueType fonts (name, file) to `doc`.
    pub fn new(doc: &mut Document, custom: Vec<(String, Vec<u8>)>) -> Result<Self> {
        let mut fonts = Fonts {
            resources: Dictionary::new(),
            widths: HashMap::new(),
        };
        for (key, base) in [
            ("Helv", "Helvetica"),
            ("TiRo", "Times-Roman"),
            ("Cour", "Courier"),
        ] {
            let font = dictionary! {
                "Type" => "Font",
                "Subtype" => "Type1",
                "BaseFont" => base,
                "Encoding" => "WinAnsiEncoding",
            };
            fonts.resources.set(key, doc.add_object(font));
            let width = |c| {
                if key == "Cour" {
                    600
                } else {
                    helvetica_width(c)
                }
            };
            fonts.widths.insert(key.into(), table(width));
        }
        let zapf =
            dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "ZapfDingbats" };
        fonts.resources.set("ZaDb", doc.add_object(zapf));

        for (name, data) in custom {
            let (font, widths) =
                embed(doc, &name, data).with_context(|| format!("font '{name}'"))?;
            fonts.resources.set(name.as_str(), font);
            fonts.widths.insert(name, widths);
        }
        Ok(fonts)
    }

    pub fn contains(&self, font: &str) -> bool {
        self.widths.contains_key(font)
    }

    /// Width of `text` in `font` at `size` pt.
    pub fn width(&self, font: &str, text: &str, size: f64) -> f64 {
        let widths = &self.widths[font];
        let units: u32 = text
            .chars()
            .map(|c| widths[encode(c).unwrap_or(b'?') as usize] as u32)
            .sum();
        units as f64 / 1000.0 * size
    }
}

/// Embed a TrueType font as simple font with WinAnsiEncoding.
fn embed(doc: &mut Document, name: &str, data: Vec<u8>) -> Result<(Object, [u16; 256])> {
    let face = ttf_parser::Face::parse(&data, 0).context("not a TrueType/OpenType font")?;
    ensure!(
        face.tables().glyf.is_some(),
        "only fonts with TrueType outlines (.ttf) are supported"
    );
    let scale = 1000.0 / face.units_per_em() as f64;
    let units = |v: i16| Object::Integer((v as f64 * scale).round() as i64);
    let widths = table(|c| {
        let advance = face.glyph_index(c).and_then(|g| face.glyph_hor_advance(g));
        advance.map_or(0, |a| (a as f64 * scale).round() as u16)
    });

    let postscript_name = face
        .names()
        .into_iter()
        .filter(|n| n.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
        .find_map(|n| n.to_string())
        .unwrap_or_else(|| name.to_string());
    let bbox = face.global_bounding_box();
    let flags =
        32 | if face.is_monospaced() { 1 } else { 0 } | if face.is_italic() { 64 } else { 0 };

    let mut descriptor = dictionary! {
        "Type" => "FontDescriptor",
        "FontName" => Object::Name(postscript_name.clone().into_bytes()),
        "Flags" => flags,
        "FontBBox" => vec![units(bbox.x_min), units(bbox.y_min), units(bbox.x_max), units(bbox.y_max)],
        "ItalicAngle" => Object::Real(face.italic_angle()),
        "Ascent" => units(face.ascender()),
        "Descent" => units(face.descender()),
        "CapHeight" => units(face.capital_height().unwrap_or(face.ascender())),
        "StemV" => 80,
    };
    let length = data.len() as i64;
    let file = doc.add_object(Stream::new(dictionary! { "Length1" => length }, data));
    descriptor.set("FontFile2", file);
    let descriptor = doc.add_object(descriptor);
    let font = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "TrueType",
        "BaseFont" => Object::Name(postscript_name.into_bytes()),
        "FirstChar" => 32,
        "LastChar" => 255,
        "Widths" => widths[32..].iter().map(|&w| Object::Integer(w as i64)).collect::<Vec<_>>(),
        "Encoding" => "WinAnsiEncoding",
        "FontDescriptor" => descriptor,
    });
    Ok((font.into(), widths))
}

/// Width table indexed by WinAnsi code.
fn table(width: impl Fn(char) -> u16) -> [u16; 256] {
    let mut widths = [0; 256];
    for (code, w) in widths.iter_mut().enumerate() {
        if let Some(c) = decode(code as u8) {
            *w = width(c);
        }
    }
    widths
}

const WINANSI_HIGH: &str = "€\u{81}‚ƒ„…†‡ˆ‰Š‹Œ\u{8d}Ž\u{8f}\u{90}‘’“”•–—˜™š›œ\u{9d}žŸ";

/// WinAnsiEncoding (CP-1252) byte of `c`.
pub fn encode(c: char) -> Option<u8> {
    match c {
        ' '..='~' | '\u{A0}'..='\u{FF}' => Some(c as u8),
        _ => WINANSI_HIGH
            .chars()
            .position(|h| h == c)
            .map(|i| 0x80 + i as u8),
    }
}

fn decode(b: u8) -> Option<char> {
    match b {
        0x20..=0x7E | 0xA0..=0xFF => Some(b as char),
        0x80..=0x9F => WINANSI_HIGH.chars().nth(b as usize - 0x80),
        _ => None,
    }
}

/// Helvetica metrics (1/1000 em) for ASCII; other characters are approximated.
fn helvetica_width(c: char) -> u16 {
    const ASCII: [u16; 95] = [
        278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278,
        278, // ' '..'/'
        556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584,
        556, // '0'..'?'
        1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722,
        778, // '@'..'O'
        667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469,
        556, // 'P'..'_'
        333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556,
        556, // '`'..'o'
        556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584, // 'p'..'~'
    ];
    match c {
        ' '..='~' => ASCII[c as usize - 32],
        'Ä' | 'Ü' => 722,
        'Ö' => 778,
        'ß' => 611,
        _ => 556,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winansi_roundtrip() {
        for b in (0x20..=0x7E).chain(0x80..=0xFF) {
            if let Some(c) = decode(b) {
                assert_eq!(encode(c), Some(b));
            }
        }
        assert_eq!(encode('—'), Some(0x97));
        assert_eq!(encode('中'), None);
    }
}
