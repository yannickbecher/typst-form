use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use std::{fs, thread};

use anyhow::{Context, Result};

use crate::form::{self, Stats};
use crate::{embedded, model};

/// Add the form to a PDF in memory. `None`: no form data embedded.
pub fn process(pdf: &[u8]) -> Result<Option<(Vec<u8>, Stats)>> {
    let mut doc = lopdf::Document::load_mem(pdf).context("failed to parse PDF")?;
    let Some(json) = embedded::take(&mut doc, "typst-form.json")? else {
        return Ok(None);
    };
    let form = model::parse(&json)?;
    let mut fonts = Vec::new();
    for name in &form.fonts {
        let file = format!("typst-form-font-{name}.ttf");
        let data =
            embedded::take(&mut doc, &file)?.with_context(|| format!("{file} is missing"))?;
        fonts.push((name.clone(), data));
    }
    let stats = form::inject(&mut doc, form, fonts)?;
    doc.compress();
    let mut out = Vec::new();
    doc.save_to(&mut out)?;
    Ok(Some((out, stats)))
}

/// Process `input` and write the result to `output`.
pub fn process_file(input: &Path, output: &Path) -> Result<Option<Stats>> {
    let pdf = fs::read(input).with_context(|| format!("failed to read {}", input.display()))?;
    let result = process(&pdf).with_context(|| input.display().to_string())?;
    let (bytes, stats) = match result {
        Some((bytes, stats)) => (bytes, Some(stats)),
        None if input == output => return Ok(None),
        None => (pdf, None),
    };
    write_atomic(output, &bytes)?;
    Ok(stats)
}

/// Re-run `on_change` whenever the file changes and has stopped changing.
pub fn watch(pdf: &Path, mut on_change: impl FnMut()) -> ! {
    let stamp = || fs::metadata(pdf).ok().map(|m| (m.modified().ok(), m.len()));
    let mut seen = None;
    let mut pending = None;
    loop {
        thread::sleep(Duration::from_millis(150));
        let now = stamp();
        if now.is_none() || now == seen {
            continue;
        }
        if now != pending {
            // Changed since the last poll: Typst may still be writing.
            pending = now;
            continue;
        }
        on_change();
        seen = stamp();
    }
}

/// Write via a temporary file and rename, so readers never see a partial PDF.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    // Process id and clock keep the name unpredictable and two runs on the same
    // file apart.
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".{}-{nonce}.tmp", std::process::id()));
    let tmp = PathBuf::from(tmp);
    // `create_new` (O_EXCL) never opens an existing file or follows a symlink
    // planted at that name. Such a file is not ours, so it is left alone.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
        .with_context(|| format!("failed to create {}", tmp.display()))?;
    if let Err(err) = file.write_all(bytes) {
        // A full disk must not leave a partial file next to the PDF.
        drop(file);
        let _ = fs::remove_file(&tmp);
        return Err(err).with_context(|| format!("failed to write {}", tmp.display()));
    }
    drop(file);
    fs::rename(&tmp, path).map_err(|err| {
        let _ = fs::remove_file(&tmp);
        anyhow::anyhow!(
            "failed to replace {} ({err}); is it locked by a PDF viewer?",
            path.display()
        )
    })
}

/// Current time as HH:MM:SS (UTC) for log lines.
pub fn clock() -> String {
    let s = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    format!("{:02}:{:02}:{:02}", s / 3600 % 24, s / 60 % 60, s % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `cli/tests/umlaute.typ` compiled by Typst, before injection.
    const FIXTURE: &[u8] = include_bytes!("../tests/umlaute.pdf");

    #[test]
    fn write_atomic_replaces_the_file_and_leaves_no_temp_file() {
        let dir = std::env::temp_dir().join(format!("typst-form-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let target = dir.join("out.pdf");
        fs::write(&target, b"old").unwrap();
        write_atomic(&target, b"new").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp files left: {leftovers:?}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn injects_fields_and_removes_attachment() {
        let (pdf, stats) = process(FIXTURE).unwrap().expect("form data embedded");
        assert_eq!((stats.fields, stats.widgets, stats.scripts), (6, 7, 0));

        // A second run finds nothing to do.
        assert!(process(&pdf).unwrap().is_none());

        let doc = lopdf::Document::load_mem(&pdf).unwrap();
        let catalog = doc.catalog().unwrap();
        assert!(catalog.get(b"Names").is_err() && catalog.get(b"AF").is_err());
        let acroform =
            doc.get_dictionary(catalog.get(b"AcroForm").unwrap().as_reference().unwrap());
        let fields = acroform
            .unwrap()
            .get(b"Fields")
            .unwrap()
            .as_array()
            .unwrap();
        let name = doc
            .get_dictionary(fields[0].as_reference().unwrap())
            .unwrap()
            .get(b"T")
            .unwrap();
        assert_eq!(lopdf::decode_text_string(name).unwrap(), "straße");

        // Typst output is tagged: the widget is in the structure tree.
        let field = doc
            .get_dictionary(fields[0].as_reference().unwrap())
            .unwrap();
        assert!(field.get(b"StructParent").is_ok());
    }

    /// `cli/tests/events.typ` compiled by Typst, before injection.
    const EVENTS_FIXTURE: &[u8] = include_bytes!("../tests/events.pdf");

    const EVENTS: [(&str, &[u8]); 10] = [
        ("keystroke", b"K"),
        ("format", b"F"),
        ("validate", b"V"),
        ("calculate", b"C"),
        ("focus", b"Fo"),
        ("blur", b"Bl"),
        ("mouse-enter", b"E"),
        ("mouse-exit", b"X"),
        ("mouse-down", b"D"),
        ("mouse-up", b"U"),
    ];

    /// The fields of a processed PDF by name (one widget each, so field and
    /// widget are the same dictionary).
    fn fields(doc: &lopdf::Document) -> std::collections::HashMap<String, lopdf::Dictionary> {
        let catalog = doc.catalog().unwrap();
        let acroform = doc
            .get_dictionary(catalog.get(b"AcroForm").unwrap().as_reference().unwrap())
            .unwrap();
        acroform
            .get(b"Fields")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                let dict = doc.get_dictionary(f.as_reference().unwrap()).unwrap();
                let name = lopdf::decode_text_string(dict.get(b"T").unwrap()).unwrap();
                (name, dict.clone())
            })
            .collect()
    }

    fn is_name(dict: &lopdf::Dictionary, key: &[u8], expected: &[u8]) -> bool {
        matches!(dict.get(key), Ok(lopdf::Object::Name(n)) if n == expected)
    }

    fn dict_of<'a>(
        doc: &'a lopdf::Document,
        parent: &'a lopdf::Dictionary,
        key: &[u8],
    ) -> &'a lopdf::Dictionary {
        let object = parent
            .get(key)
            .unwrap_or_else(|_| panic!("no /{}", String::from_utf8_lossy(key)));
        doc.dereference(object).unwrap().1.as_dict().unwrap()
    }

    /// The JavaScript of `/AA /<key>`, if any.
    fn event_script(
        doc: &lopdf::Document,
        field: &lopdf::Dictionary,
        key: &[u8],
    ) -> Option<String> {
        let aa = dict_of(doc, field, b"AA");
        let action = dict_of(doc, aa, key);
        assert!(is_name(action, b"S", b"JavaScript"));
        Some(lopdf::decode_text_string(action.get(b"JS").ok()?).unwrap())
    }

    #[test]
    fn every_event_reaches_every_field_type() {
        let (pdf, _) = process(EVENTS_FIXTURE)
            .unwrap()
            .expect("form data embedded");
        let doc = lopdf::Document::load_mem(&pdf).unwrap();
        let fields = fields(&doc);
        for kind in [
            "text",
            "check",
            "radio",
            "combo",
            "list",
            "button",
            "signature",
        ] {
            let field = &fields[kind];
            for (event, key) in EVENTS {
                let key_name = String::from_utf8_lossy(key);
                assert_eq!(
                    event_script(&doc, field, key).as_deref(),
                    Some(format!("/*{kind}:{event}*/").as_str()),
                    "{kind}: event {event} (/AA /{key_name})"
                );
            }
        }
    }

    #[test]
    fn signature_field_is_a_signature_with_a_lock() {
        let (pdf, _) = process(EVENTS_FIXTURE)
            .unwrap()
            .expect("form data embedded");
        let doc = lopdf::Document::load_mem(&pdf).unwrap();
        let sig = &fields(&doc)["signature"];
        assert!(is_name(sig, b"FT", b"Sig"));
        let lock = dict_of(&doc, sig, b"Lock");
        assert!(is_name(lock, b"Action", b"Include"));
        assert_eq!(lock.get(b"Fields").unwrap().as_array().unwrap().len(), 2);
    }

    #[test]
    fn button_actions_become_pdf_actions() {
        let (pdf, _) = process(EVENTS_FIXTURE)
            .unwrap()
            .expect("form data embedded");
        let doc = lopdf::Document::load_mem(&pdf).unwrap();
        let fields = fields(&doc);
        let action = |name: &str| dict_of(&doc, &fields[name], b"A");
        let script =
            |name: &str| lopdf::decode_text_string(action(name).get(b"JS").unwrap()).unwrap();

        assert!(is_name(action("act-js"), b"S", b"JavaScript"));
        assert_eq!(script("act-js"), "1 + 1;");
        assert_eq!(script("act-print"), "this.print();");

        let url = action("act-url");
        assert!(is_name(url, b"S", b"URI"));
        assert!(
            matches!(url.get(b"URI"), Ok(lopdf::Object::String(s, _)) if s == b"https://example.com/")
        );

        // Flags: empty (bit 2) + xfdf (bit 6) + get (bit 4) = 2 + 32 + 8.
        let submit = action("act-submit");
        assert!(is_name(submit, b"S", b"SubmitForm"));
        assert_eq!(submit.get(b"Flags").unwrap().as_i64().unwrap(), 42);
        assert_eq!(submit.get(b"Fields").unwrap().as_array().unwrap().len(), 1);
        assert!(is_name(dict_of(&doc, submit, b"F"), b"FS", b"URL"));

        let reset = action("act-reset");
        assert!(is_name(reset, b"S", b"ResetForm"));
        assert_eq!(reset.get(b"Fields").unwrap().as_array().unwrap().len(), 2);

        for (name, hide) in [("act-hide", true), ("act-show", false)] {
            let a = action(name);
            assert!(is_name(a, b"S", b"Hide"), "{name}");
            assert_eq!(a.get(b"H").unwrap().as_bool().unwrap(), hide, "{name}");
        }
    }
}
