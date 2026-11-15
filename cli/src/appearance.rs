//! Appearance streams. Frames and backgrounds are drawn by Typst as page
//! content, so a widget only paints what it shows: text, a symbol, a list.

use crate::fonts::{self, Fonts};
use crate::model::{Align, Kind, Style, Widget};

const PAD: f64 = 2.0;

/// Content stream for the widget's normal appearance (`None`: nothing to draw).
pub fn normal(w: &Widget, fonts: &Fonts) -> Option<String> {
    let width = |text: &str| fonts.width(&w.font, text, w.font_size);
    let lines: Vec<String> = match w.kind {
        Kind::Text if w.password => vec!["*".repeat(w.values().concat().chars().count())],
        Kind::Text if w.multiline => wrap(&w.values().concat(), w.width - 2.0 * PAD, width),
        Kind::Text => vec![w.values().concat()],
        Kind::Combo => {
            let value = w.values().concat();
            let label = w
                .options
                .iter()
                .find(|o| o.value == value)
                .map(|o| &o.label);
            vec![label.cloned().unwrap_or(value)]
        }
        Kind::List => return Some(list(w)),
        _ => return None,
    };
    if lines.concat().is_empty() {
        return None;
    }

    let mut ops = clip(w) + &font(w);
    if w.kind == Kind::Text && w.comb {
        // One character per cell.
        let cell = w.width / w.max_len.unwrap_or(1).max(1) as f64;
        for (i, c) in lines[0].chars().enumerate() {
            let s = c.to_string();
            let x = cell * i as f64 + (cell - width(&s)) / 2.0;
            ops += &format!("1 0 0 1 {x:.2} {:.2} Tm {}\n", baseline(w), show(&s));
        }
    } else if w.multiline {
        let leading = w.font_size * 1.15;
        let top = w.height - PAD - w.font_size;
        for (i, line) in lines.iter().enumerate() {
            let y = top - leading * i as f64;
            ops += &format!(
                "1 0 0 1 {:.2} {y:.2} Tm {}\n",
                x(w, width(line)),
                show(line)
            );
        }
    } else {
        let line = &lines[0];
        ops += &format!(
            "1 0 0 1 {:.2} {:.2} Tm {}\n",
            x(w, width(line)),
            baseline(w),
            show(line)
        );
    }
    Some(ops + "ET Q EMC")
}

/// Content stream of the "on" state of a checkbox or radio button: the
/// symbol as vector path, so it does not depend on the viewer's fonts.
pub fn on(w: &Widget) -> String {
    let s = w.width.min(w.height);
    let (cx, cy) = (w.width / 2.0, w.height / 2.0);
    let p = |x: f64, y: f64| format!("{:.2} {:.2}", cx + x * s, cy + y * s);
    let polygon = |points: &[(f64, f64)]| {
        let mut path = String::new();
        for (i, &(x, y)) in points.iter().enumerate() {
            path += &format!("{} {} ", p(x, y), if i == 0 { "m" } else { "l" });
        }
        path + "f"
    };
    let stroke = |path: String| format!("{:.2} w 1 J 1 j {path} S", s * 0.12);
    let shape = match w.style {
        Style::Check => stroke(format!(
            "{} m {} l {} l",
            p(-0.28, 0.0),
            p(-0.08, -0.22),
            p(0.3, 0.24)
        )),
        Style::Cross => stroke(format!(
            "{} m {} l {} m {} l",
            p(-0.25, -0.25),
            p(0.25, 0.25),
            p(-0.25, 0.25),
            p(0.25, -0.25)
        )),
        Style::Circle => {
            // Four Bézier arcs, k = 4/3 (sqrt 2 - 1).
            let (r, k) = (0.25, 0.25 * 0.5523);
            format!(
                "{} m {} {} {} c {} {} {} c {} {} {} c {} {} {} c f",
                p(r, 0.0),
                p(r, k),
                p(k, r),
                p(0.0, r),
                p(-k, r),
                p(-r, k),
                p(-r, 0.0),
                p(-r, -k),
                p(-k, -r),
                p(0.0, -r),
                p(k, -r),
                p(r, -k),
                p(r, 0.0),
            )
        }
        Style::Square => polygon(&[(-0.22, -0.22), (0.22, -0.22), (0.22, 0.22), (-0.22, 0.22)]),
        Style::Diamond => polygon(&[(0.0, 0.3), (0.3, 0.0), (0.0, -0.3), (-0.3, 0.0)]),
        Style::Star => {
            let points: Vec<_> = (0..10)
                .map(|i| {
                    let r = if i % 2 == 0 { 0.32 } else { 0.13 };
                    let a = std::f64::consts::FRAC_PI_2 + i as f64 * std::f64::consts::PI / 5.0;
                    (r * a.cos(), r * a.sin())
                })
                .collect();
            polygon(&points)
        }
    };
    let [r, g, b] = w.text_color;
    format!("q {r:.3} {g:.3} {b:.3} RG {} {shape} Q", rgb(w))
}

fn list(w: &Widget) -> String {
    let leading = w.font_size * 1.15;
    let selected = w.values();
    let mut ops = clip(w);
    for (i, option) in w.options.iter().enumerate() {
        let top = w.height - PAD - leading * i as f64;
        if top - leading < 0.0 {
            break;
        }
        if selected.contains(&option.value.as_str()) {
            ops += &format!(
                "0.6 0.75 0.86 rg 0 {:.2} {:.2} {leading:.2} re f\n",
                top - leading,
                w.width
            );
        }
    }
    ops += &font(w);
    for (i, option) in w.options.iter().enumerate() {
        let y = w.height - PAD - leading * i as f64 - w.font_size;
        if y < 0.0 {
            break;
        }
        ops += &format!("1 0 0 1 {PAD:.2} {y:.2} Tm {}\n", show(&option.label));
    }
    ops + "ET Q EMC"
}

/// Marked content for variable text, clipped to the inner area.
fn clip(w: &Widget) -> String {
    let (cw, ch) = (w.width - 2.0 * PAD, w.height - 2.0 * PAD);
    format!("/Tx BMC q\n{PAD:.2} {PAD:.2} {cw:.2} {ch:.2} re W n\n")
}

fn font(w: &Widget) -> String {
    format!("BT /{} {:.2} Tf {}\n", w.font, w.font_size, rgb(w))
}

/// Fill color operator for the widget's text color.
pub fn rgb(w: &Widget) -> String {
    let [r, g, b] = w.text_color;
    format!("{r:.3} {g:.3} {b:.3} rg")
}

/// Baseline that centers a single line vertically.
fn baseline(w: &Widget) -> f64 {
    (w.height - w.font_size * 0.7) / 2.0
}

/// Start of a line of the given width, according to the alignment.
fn x(w: &Widget, line_width: f64) -> f64 {
    let free = w.width - 2.0 * PAD - line_width;
    PAD + match w.align {
        Align::Left => 0.0,
        Align::Center => free / 2.0,
        Align::Right => free,
    }
}

/// Greedy word wrap of each paragraph to `width`.
fn wrap(text: &str, max: f64, width: impl Fn(&str) -> f64) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split(' ') {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if width(&candidate) > max && !line.is_empty() {
                lines.push(std::mem::replace(&mut line, word.to_string()));
            } else {
                line = candidate;
            }
        }
        lines.push(line);
    }
    lines
}

/// `(text) Tj` in WinAnsiEncoding; octal escapes keep the stream ASCII.
fn show(text: &str) -> String {
    let mut out = String::from("(");
    for c in text.chars() {
        match c {
            '(' | ')' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            ' '..='~' => out.push(c),
            _ => out += &format!("\\{:03o}", fonts::encode(c).unwrap_or(b'?')),
        }
    }
    out + ") Tj"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_escapes_and_encodes() {
        assert_eq!(show("a(b)\\"), r"(a\(b\)\\) Tj");
        assert_eq!(show("ü€中"), r"(\374\200\077) Tj");
    }
}
