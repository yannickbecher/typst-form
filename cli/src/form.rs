//! Turn widgets into an AcroForm (ISO 32000-1, section 12.7).
//!
//! Widgets with the same name become one field (radio groups, mirrored
//! fields); dots in names build a field hierarchy (`person.name`).

use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, Result, ensure};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream, dictionary, text_string};

use crate::fonts::Fonts;
use crate::model::{Action, Align, Form, Kind, Lock, Script, Widget};
use crate::{appearance, tagging};

/// What was added to the document.
#[derive(Debug, Default, Clone, Copy)]
pub struct Stats {
    pub fields: usize,
    pub widgets: usize,
    pub scripts: usize,
}

/// Events that belong to the field (`/AA` of the field dictionary).
const FIELD_EVENTS: [(&str, &str); 4] = [
    ("keystroke", "K"),
    ("format", "F"),
    ("validate", "V"),
    ("calculate", "C"),
];
/// Events that belong to a single widget annotation.
const WIDGET_EVENTS: [(&str, &str); 6] = [
    ("focus", "Fo"),
    ("blur", "Bl"),
    ("mouse-enter", "E"),
    ("mouse-exit", "X"),
    ("mouse-down", "D"),
    ("mouse-up", "U"),
];

/// Add the form to `doc`. `fonts`: files of the fonts named in `form.fonts`.
pub fn inject(doc: &mut Document, form: Form, fonts: Vec<(String, Vec<u8>)>) -> Result<Stats> {
    let Form {
        widgets,
        mut scripts,
        ..
    } = form;
    let fonts = Fonts::new(doc, fonts)?;
    if let Some(w) = widgets.iter().find(|w| !fonts.contains(&w.font)) {
        anyhow::bail!(
            "field '{}' uses font '{}', which is not registered with form-font",
            w.name,
            w.font
        );
    }
    ensure!(
        doc.catalog()?.get(b"AcroForm").is_err(),
        "the PDF already contains a form"
    );
    let stats = Stats {
        fields: 0,
        widgets: widgets.len(),
        scripts: scripts.len(),
    };

    // Group widgets into fields by name, keeping document order.
    let mut groups: Vec<Vec<Widget>> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for w in widgets {
        if let Some(&i) = index.get(&w.name) {
            ensure!(
                groups[i][0].kind == w.kind,
                "field '{}' is used with different types",
                w.name
            );
            groups[i].push(w);
        } else {
            index.insert(w.name.clone(), groups.len());
            groups.push(vec![w]);
        }
    }

    let pages = doc.get_pages();
    let mut tops = HashMap::new();
    let mut tree = Tree::default();
    let mut annots: BTreeMap<ObjectId, Vec<(i64, usize, ObjectId)>> = BTreeMap::new();
    let mut calculation_order = Vec::new();
    let mut seq = 0;

    for group in &groups {
        let first = &group[0];
        let field_id = doc.new_object_id();
        let mut field = field_dict(first, group);
        let mut kids = Vec::new();

        for w in group {
            let page = *pages
                .get(&w.page)
                .with_context(|| format!("page {} not found", w.page))?;
            let top = match tops.get(&page) {
                Some(&top) => top,
                None => {
                    let top = media_box_top(doc, page)?;
                    tops.insert(page, top);
                    top
                }
            };
            let widget = widget_dict(doc, w, page, top, &fonts);
            let id = if group.len() == 1 {
                merge(&mut field, widget);
                field_id
            } else {
                let id = doc.add_object(dictionary! { "Parent" => field_id });
                merge(doc.get_dictionary_mut(id)?, widget);
                kids.push(Object::Reference(id));
                id
            };
            annots
                .entry(page)
                .or_default()
                .push((w.tab_order.unwrap_or(i64::MAX), seq, id));
            seq += 1;
        }

        if !kids.is_empty() {
            field.set("Kids", kids);
        }
        if first.actions.contains_key("calculate") {
            calculation_order.push(Object::Reference(field_id));
        }
        tree.add(doc, &first.name, field_id, &mut field);
        doc.objects.insert(field_id, Object::Dictionary(field));
    }

    // Annotation order (and structure order) is the tab order.
    let mut ordered = Vec::new();
    for (page, mut list) in annots {
        list.sort();
        ordered.extend(list.iter().map(|&(_, _, id)| (id, page)));
        append_annots(
            doc,
            page,
            list.into_iter().map(|(_, _, id)| id.into()).collect(),
        )?;
    }
    tagging::tag(doc, &ordered)?;

    let mut acroform = dictionary! {
        "Fields" => tree.finish(doc, &index)?,
        "DA" => text_string("/Helv 0 Tf 0 g"),
        "DR" => dictionary! { "Font" => fonts.resources },
    };
    if !calculation_order.is_empty() {
        acroform.set("CO", calculation_order);
        // Fill calculated fields on open; sorts after the document's own scripts.
        scripts.push(Script {
            name: "~typst-form-calculate".into(),
            code: "this.calculateNow();".into(),
        });
    }
    let acroform = doc.add_object(acroform);
    doc.catalog_mut()?.set("AcroForm", acroform);

    if !scripts.is_empty() {
        add_scripts(doc, &scripts)?;
    }
    Ok(Stats {
        fields: groups.len(),
        ..stats
    })
}

/// The field dictionary: type, flags, value and field-level events.
fn field_dict(w: &Widget, group: &[Widget]) -> Dictionary {
    let mut ff = bit(w.read_only, 1) | bit(w.required, 2) | bit(w.no_export, 3);
    let mut d = Dictionary::new();
    let mut font = w.font.as_str();

    match w.kind {
        Kind::Text => {
            d.set("FT", "Tx");
            ff |= bit(w.multiline, 13)
                | bit(w.password, 14)
                | bit(w.file_select, 21)
                | bit(!w.spell_check, 23)
                | bit(!w.scroll, 24)
                | bit(w.comb, 25);
            if let Some(max_len) = w.max_len {
                d.set("MaxLen", max_len);
            }
            d.set("Q", quadding(w.align));
            // ISO 32000-1 12.7.4.3: a password value must not be stored in the file.
            if !w.password {
                set_value(&mut d, &w.values());
            }
        }
        Kind::Checkbox | Kind::Radio => {
            d.set("FT", "Btn");
            if w.kind == Kind::Radio {
                ff |= bit(!w.allow_off, 15) | bit(true, 16) | bit(w.unison, 26);
            }
            let on = group
                .iter()
                .find(|w| w.checked)
                .map_or("Off", Widget::export);
            d.set("V", name(on));
            d.set("DV", name(on));
            font = "ZaDb";
        }
        Kind::Combo | Kind::List => {
            d.set("FT", "Ch");
            ff |= bit(w.kind == Kind::Combo, 18)
                | bit(w.editable, 19)
                | bit(w.sort, 20)
                | bit(w.multi_select, 22)
                | bit(w.kind == Kind::Combo && !w.spell_check, 23)
                | bit(w.commit_on_change, 27);
            let options = w.options.iter().map(|o| {
                if o.value == o.label {
                    text_string(&o.label)
                } else {
                    vec![text_string(&o.value), text_string(&o.label)].into()
                }
            });
            d.set("Opt", options.collect::<Vec<_>>());
            let values = w.values();
            set_value(&mut d, &values);
            let selected: Vec<Object> = (w.options.iter().enumerate())
                .filter(|(_, o)| values.contains(&o.value.as_str()))
                .map(|(i, _)| (i as i64).into())
                .collect();
            if w.kind == Kind::List && !selected.is_empty() {
                d.set("I", selected);
            }
        }
        Kind::Button => {
            d.set("FT", "Btn");
            ff |= bit(true, 17);
        }
        Kind::Signature => {
            d.set("FT", "Sig");
            if let Some(lock) = &w.lock {
                let mut l = dictionary! { "Type" => "SigFieldLock", "Action" => "All" };
                if let Lock::Fields(fields) = lock {
                    l.set("Action", "Include");
                    l.set("Fields", names(fields));
                }
                d.set("Lock", l);
            }
        }
    }

    d.set("Ff", ff);
    let size = if font == "ZaDb" { 0.0 } else { w.font_size };
    d.set(
        "DA",
        text_string(&format!("/{font} {size:.2} Tf {}", appearance::rgb(w))),
    );
    if let Some(tooltip) = &w.tooltip {
        d.set("TU", text_string(tooltip));
    }
    set_events(&mut d, w, &FIELD_EVENTS);
    d
}

/// The widget annotation: position, appearance, action and widget events.
fn widget_dict(
    doc: &mut Document,
    w: &Widget,
    page: ObjectId,
    top: f64,
    fonts: &Fonts,
) -> Dictionary {
    let rect = [w.x, top - w.y - w.height, w.x + w.width, top - w.y];
    let mut d = dictionary! {
        "Type" => "Annot",
        "Subtype" => "Widget",
        "Rect" => rect.map(|v| Object::Real(v as f32)).to_vec(),
        "P" => page,
        "F" => if w.hidden { 2 } else { 4 }, // Hidden or Print
    };

    if matches!(w.kind, Kind::Checkbox | Kind::Radio) {
        let on = xobject(doc, w, fonts, appearance::on(w));
        let off = xobject(doc, w, fonts, String::new());
        let state = if w.checked { w.export() } else { "Off" };
        d.set(
            "AP",
            dictionary! { "N" => dictionary! { w.export() => on, "Off" => off } },
        );
        d.set("AS", name(state));
        d.set(
            "MK",
            dictionary! { "CA" => Object::string_literal(vec![w.style.glyph()]) },
        );
    } else {
        let normal = xobject(
            doc,
            w,
            fonts,
            appearance::normal(w, fonts).unwrap_or_default(),
        );
        d.set("AP", dictionary! { "N" => normal });
    }

    if let Some(action) = &w.action {
        d.set("A", action_dict(action));
    }
    set_events(&mut d, w, &WIDGET_EVENTS);
    d
}

fn action_dict(action: &Action) -> Dictionary {
    match action {
        Action::Js { script } => js(script),
        Action::Url { url } => {
            dictionary! { "S" => "URI", "URI" => Object::string_literal(url.as_str()) }
        }
        Action::Hide { fields, hide } => {
            dictionary! { "S" => "Hide", "T" => names(fields), "H" => *hide }
        }
        Action::Reset { fields } => {
            let mut d = dictionary! { "S" => "ResetForm" };
            if !fields.is_empty() {
                d.set("Fields", names(fields));
            }
            d
        }
        Action::Submit {
            url,
            format,
            method,
            fields,
            empty,
        } => {
            let flags = bit(*empty, 2)
                | bit(format == "html", 3)
                | bit(method == "get", 4)
                | bit(format == "xfdf", 6)
                | bit(format == "pdf", 9);
            let mut d = dictionary! {
                "S" => "SubmitForm",
                "F" => dictionary! { "FS" => "URL", "F" => Object::string_literal(url.as_str()) },
                "Flags" => flags,
            };
            if !fields.is_empty() {
                d.set("Fields", names(fields));
            }
            d
        }
    }
}

/// Builds the hierarchy of non-terminal fields for dotted names.
#[derive(Default)]
struct Tree {
    roots: Vec<Object>,
    nodes: HashMap<String, (ObjectId, Vec<Object>)>,
}

impl Tree {
    /// Place the field `id` with full name `name`; sets its /T and /Parent.
    fn add(&mut self, doc: &mut Document, name: &str, id: ObjectId, dict: &mut Dictionary) {
        let Some((parent, partial)) = name.rsplit_once('.') else {
            dict.set("T", text_string(name));
            self.roots.push(id.into());
            return;
        };
        dict.set("T", text_string(partial));
        let parent_id = self.node(doc, parent);
        dict.set("Parent", parent_id);
        self.nodes
            .get_mut(parent)
            .expect("node exists")
            .1
            .push(id.into());
    }

    fn node(&mut self, doc: &mut Document, path: &str) -> ObjectId {
        if let Some((id, _)) = self.nodes.get(path) {
            return *id;
        }
        let id = doc.new_object_id();
        self.nodes.insert(path.to_string(), (id, Vec::new()));
        let mut dict = Dictionary::new();
        self.add(doc, path, id, &mut dict);
        doc.objects.insert(id, dict.into());
        id
    }

    /// Write the /Kids of all nodes and return the root fields.
    fn finish(self, doc: &mut Document, terminals: &HashMap<String, usize>) -> Result<Vec<Object>> {
        for (path, (id, kids)) in self.nodes {
            ensure!(
                !terminals.contains_key(&path),
                "'{path}' is used as a field and as a group"
            );
            doc.get_dictionary_mut(id)?.set("Kids", kids);
        }
        Ok(self.roots)
    }
}

/// Add document-level JavaScript (`/Names /JavaScript`).
fn add_scripts(doc: &mut Document, scripts: &[Script]) -> Result<()> {
    let mut sorted: Vec<_> = scripts.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    let mut entries = Vec::new();
    for script in sorted {
        entries.push(Object::string_literal(script.name.as_str()));
        entries.push(doc.add_object(js(&script.code)).into());
    }
    let tree = doc.add_object(dictionary! { "Names" => entries });
    let catalog = doc.catalog_mut()?;
    match catalog.get_mut(b"Names").and_then(Object::as_dict_mut) {
        Ok(names) => names.set("JavaScript", tree),
        Err(_) => catalog.set("Names", dictionary! { "JavaScript" => tree }),
    }
    Ok(())
}

fn xobject(doc: &mut Document, w: &Widget, fonts: &Fonts, content: String) -> ObjectId {
    let dict = dictionary! {
        "Type" => "XObject",
        "Subtype" => "Form",
        "BBox" => vec![0.into(), 0.into(), Object::Real(w.width as f32), Object::Real(w.height as f32)],
        "Resources" => dictionary! { "Font" => fonts.resources.clone() },
    };
    doc.add_object(Stream::new(dict, content.into_bytes()))
}

/// Upper edge of the page's (possibly inherited) /MediaBox; Typst measures y from there.
fn media_box_top(doc: &Document, page: ObjectId) -> Result<f64> {
    let mut dict = doc.get_dictionary(page)?;
    // Bounded: a hostile /Parent cycle must not hang the tool.
    for _ in 0..64 {
        if let Ok(media_box) = dict.get(b"MediaBox") {
            let media_box = doc.dereference(media_box)?.1.as_array()?;
            let y = |i: usize| -> Result<f32> {
                Ok(media_box.get(i).context("invalid /MediaBox")?.as_float()?)
            };
            return Ok(y(1)?.max(y(3)?) as f64);
        }
        dict = doc.get_dictionary(dict.get(b"Parent")?.as_reference()?)?;
    }
    anyhow::bail!("page without /MediaBox")
}

fn append_annots(doc: &mut Document, page: ObjectId, mut new: Vec<Object>) -> Result<()> {
    let existing = doc.get_dictionary(page)?.get(b"Annots").ok().cloned();
    let annots = match existing {
        Some(Object::Reference(id)) => doc.get_object_mut(id)?,
        Some(_) => doc.get_dictionary_mut(page)?.get_mut(b"Annots")?,
        None => {
            doc.get_dictionary_mut(page)?.set("Annots", new);
            return Ok(());
        }
    };
    annots.as_array_mut()?.append(&mut new);
    Ok(())
}

/// Add `widget` to `field`, merging the two /AA dictionaries.
fn merge(field: &mut Dictionary, widget: Dictionary) {
    for (key, value) in widget {
        match (field.get_mut(&key), value) {
            (Ok(Object::Dictionary(aa)), Object::Dictionary(more)) if key == b"AA" => {
                aa.extend(&more)
            }
            (_, value) => field.set(key, value),
        }
    }
}

fn set_events(d: &mut Dictionary, w: &Widget, events: &[(&str, &str)]) {
    let mut aa = Dictionary::new();
    for (event, key) in events {
        if let Some(script) = w.actions.get(*event) {
            aa.set(*key, js(script));
        }
    }
    if !aa.is_empty() {
        d.set("AA", aa);
    }
}

fn set_value(d: &mut Dictionary, values: &[&str]) {
    let value: Object = match values {
        [] | [""] => return,
        [v] => text_string(v),
        many => many
            .iter()
            .map(|v| text_string(v))
            .collect::<Vec<_>>()
            .into(),
    };
    d.set("V", value.clone());
    d.set("DV", value);
}

fn js(script: &str) -> Dictionary {
    dictionary! { "S" => "JavaScript", "JS" => text_string(script) }
}

fn names(fields: &[String]) -> Vec<Object> {
    fields.iter().map(|f| text_string(f)).collect()
}

fn name(s: &str) -> Object {
    Object::Name(s.as_bytes().to_vec())
}

fn quadding(align: Align) -> i64 {
    match align {
        Align::Left => 0,
        Align::Center => 1,
        Align::Right => 2,
    }
}

/// Flag bit `n` (1-based, as in the PDF specification).
fn bit(on: bool, n: u32) -> i64 {
    if on { 1 << (n - 1) } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A /Parent cycle without any /MediaBox used to loop forever.
    #[test]
    fn media_box_search_stops_on_a_parent_cycle() {
        let mut doc = Document::with_version("1.7");
        let page = doc.new_object_id();
        doc.objects.insert(
            page,
            dictionary! { "Type" => "Page", "Parent" => page }.into(),
        );
        assert!(media_box_top(&doc, page).is_err());
    }

    #[test]
    fn media_box_is_inherited() {
        let mut doc = Document::with_version("1.7");
        let parent = doc.add_object(dictionary! {
            "Type" => "Pages",
            "MediaBox" => vec![0.into(), 0.into(), 200.into(), 300.into()],
        });
        let page = doc.add_object(dictionary! { "Type" => "Page", "Parent" => parent });
        assert_eq!(media_box_top(&doc, page).unwrap(), 300.0);
    }
}
