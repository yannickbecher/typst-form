//! The JSON written by the Typst package (`core.typ`).

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Document-level JavaScript (`document-script`).
#[derive(Deserialize)]
pub struct Script {
    pub name: String,
    pub code: String,
}

/// Everything in `typst-form.json`.
#[derive(Default)]
pub struct Form {
    pub widgets: Vec<Widget>,
    pub scripts: Vec<Script>,
    /// Names of fonts registered with `form-font` (attached as files).
    pub fonts: Vec<String>,
}

/// Parse `typst-form.json`.
pub fn parse(json: &[u8]) -> Result<Form> {
    let entries: Vec<serde_json::Value> =
        serde_json::from_slice(json).context("invalid form data")?;
    let mut form = Form::default();
    for (i, entry) in entries.into_iter().enumerate() {
        let context = || format!("form entry #{}", i + 1);
        match entry["type"].as_str() {
            Some("script") => form
                .scripts
                .push(serde_json::from_value(entry).with_context(context)?),
            Some("font") => form
                .fonts
                .push(entry["name"].as_str().with_context(context)?.into()),
            _ => {
                let widget: Widget = serde_json::from_value(entry).with_context(context)?;
                widget.validate().with_context(context)?;
                form.widgets.push(widget);
            }
        }
    }
    Ok(form)
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    Text,
    Checkbox,
    Radio,
    Combo,
    List,
    Button,
    Signature,
}

/// A widget annotation. Widgets with the same name form one field.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Widget {
    #[serde(rename = "type")]
    pub kind: Kind,
    pub name: String,
    pub page: u32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,

    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub no_export: bool,
    #[serde(default)]
    pub hidden: bool,
    pub tooltip: Option<String>,
    pub tab_order: Option<i64>,
    /// JavaScript per event (`keystroke`, `format`, `focus`, ...).
    #[serde(default)]
    pub actions: BTreeMap<String, String>,
    #[serde(default = "helv")]
    pub font: String,
    #[serde(default = "ten")]
    pub font_size: f64,
    #[serde(default)]
    pub text_color: [f64; 3],

    // Text, combo, list
    #[serde(default)]
    pub value: serde_json::Value,
    #[serde(default)]
    pub multiline: bool,
    #[serde(default)]
    pub password: bool,
    #[serde(default)]
    pub file_select: bool,
    #[serde(default)]
    pub comb: bool,
    pub max_len: Option<i64>,
    #[serde(default)]
    pub align: Align,
    #[serde(default = "yes")]
    pub spell_check: bool,
    #[serde(default = "yes")]
    pub scroll: bool,

    // Checkbox, radio
    #[serde(default)]
    pub checked: bool,
    pub export: Option<String>,
    #[serde(default)]
    pub style: Style,
    #[serde(default)]
    pub allow_off: bool,
    #[serde(default)]
    pub unison: bool,

    // Combo, list
    #[serde(default)]
    pub options: Vec<Choice>,
    #[serde(default)]
    pub editable: bool,
    #[serde(default)]
    pub sort: bool,
    #[serde(default)]
    pub commit_on_change: bool,
    #[serde(default)]
    pub multi_select: bool,

    // Button, signature
    pub action: Option<Action>,
    pub lock: Option<Lock>,
}

impl Widget {
    /// Reject data that would produce a broken or hostile PDF: non-finite or
    /// absurd geometry, endless field names, export values that collide with
    /// the "Off" state.
    pub fn validate(&self) -> Result<()> {
        for (what, v) in [
            ("x", self.x),
            ("y", self.y),
            ("width", self.width),
            ("height", self.height),
            ("font-size", self.font_size),
        ] {
            anyhow::ensure!(
                v.is_finite() && v.abs() < 1e7,
                "{what} of field '{}' is not a sane number: {v}",
                self.name
            );
        }
        anyhow::ensure!(
            self.width > 0.0 && self.height > 0.0 && self.font_size > 0.0,
            "field '{}' needs a positive size",
            self.name
        );
        anyhow::ensure!(self.page >= 1, "field '{}' is on page 0", self.name);
        anyhow::ensure!(
            !self.name.is_empty() && self.name.len() <= 512 && self.name.matches('.').count() <= 32,
            "field name '{}...' is empty or too long (512 bytes, 32 dots)",
            self.name.chars().take(40).collect::<String>()
        );
        if matches!(self.kind, Kind::Checkbox | Kind::Radio) {
            let export = self.export();
            anyhow::ensure!(
                !export.is_empty() && export != "Off",
                "export value of '{}' must not be empty or \"Off\"",
                self.name
            );
        }
        Ok(())
    }

    /// The value(s) as strings: text, selected option(s).
    pub fn values(&self) -> Vec<&str> {
        match &self.value {
            serde_json::Value::String(s) => vec![s.as_str()],
            serde_json::Value::Array(a) => a.iter().filter_map(|v| v.as_str()).collect(),
            _ => vec![],
        }
    }

    pub fn export(&self) -> &str {
        self.export.as_deref().unwrap_or("Yes")
    }
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// Checkbox/radio symbol.
#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    Check,
    Circle,
    #[default]
    Cross,
    Diamond,
    Square,
    Star,
}

impl Style {
    /// ZapfDingbats character for /MK /CA. Acrobat redraws the symbol with it
    /// whenever it rebuilds the appearance, instead of using our straight
    /// vector symbol. So the characters are the plainest ones: the light check
    /// mark (not the heavy, calligraphic one) and the straight heavy X (not
    /// the ballot X). ZapfDingbats has no straight check mark.
    pub fn glyph(self) -> u8 {
        match self {
            Style::Check => b'3',
            Style::Circle => b'l',
            Style::Cross => b'6',
            Style::Diamond => b'u',
            Style::Square => b'n',
            Style::Star => b'H',
        }
    }
}

#[derive(Deserialize)]
pub struct Choice {
    pub value: String,
    pub label: String,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Action {
    Js {
        script: String,
    },
    Submit {
        url: String,
        format: String,
        method: String,
        fields: Vec<String>,
        empty: bool,
    },
    Reset {
        fields: Vec<String>,
    },
    Url {
        url: String,
    },
    Hide {
        fields: Vec<String>,
        hide: bool,
    },
}

/// Fields locked when a signature field is signed.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Lock {
    Fields(Vec<String>),
    /// `"all"`
    All(serde::de::IgnoredAny),
}

fn helv() -> String {
    "Helv".into()
}

fn ten() -> f64 {
    10.0
}

fn yes() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_widgets_and_scripts() {
        let json = br#"[
            {"type": "script", "name": "s", "code": "1;"},
            {"type": "radio", "name": "g", "export": "a", "page": 1, "x": 0, "y": 0, "width": 9, "height": 9},
            {"type": "list", "name": "l", "value": ["x", "y"], "options": [], "page": 1, "x": 0, "y": 0, "width": 9, "height": 9},
            {"type": "signature", "name": "sig", "lock": "all", "page": 1, "x": 0, "y": 0, "width": 9, "height": 9}
        ]"#;
        let Form {
            widgets, scripts, ..
        } = parse(json).unwrap();
        assert_eq!(scripts[0].code, "1;");
        assert_eq!(widgets[0].kind, Kind::Radio);
        assert_eq!(widgets[0].export(), "a");
        assert_eq!(widgets[1].values(), ["x", "y"]);
        assert!(matches!(widgets[2].lock, Some(Lock::All(_))));
        assert!(widgets[0].spell_check);
    }

    fn parse_one(extra: &str) -> Result<Form> {
        let json = format!(
            r#"[{{"type": "text", "name": "a", "page": 1, "x": 0, "y": 0, "width": 10, "height": 10 {extra}}}]"#
        );
        parse(json.as_bytes())
    }

    #[test]
    fn rejects_data_that_would_break_the_pdf() {
        assert!(parse_one("").is_ok());
        for bad in [
            r#", "width": 1e300"#,
            r#", "width": 0"#,
            r#", "height": -5"#,
            r#", "font-size": 0"#,
            r#", "page": 0"#,
            r#", "name": """#,
        ] {
            assert!(parse_one(bad).is_err(), "{bad} must be rejected");
        }
        let long = format!(r#", "name": "{}""#, "a.".repeat(40));
        assert!(parse_one(&long).is_err(), "too many dots");
        let long = format!(r#", "name": "{}""#, "a".repeat(600));
        assert!(parse_one(&long).is_err(), "name too long");
    }

    #[test]
    fn checkbox_export_must_not_collide_with_off() {
        let bad = |export: &str| {
            let json = format!(
                r#"[{{"type": "checkbox", "name": "c", "export": "{export}", "page": 1, "x": 0, "y": 0, "width": 9, "height": 9}}]"#
            );
            parse(json.as_bytes()).is_err()
        };
        assert!(bad("Off") && bad(""));
        assert!(!bad("Yes"));
    }
}
