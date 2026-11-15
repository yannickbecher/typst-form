//! Take attachments (`typst-form.json`, fonts) out of a PDF written by Typst.
//!
//! Typst (krilla) writes `/Names << /EmbeddedFiles << /Names [..] >> >>` and
//! `/AF [..]` inline into the catalog, so that is all we need to handle.

use anyhow::{Context, Result};
use lopdf::{Document, Object};

/// Largest attachment (form data or font) we are willing to unpack.
const MAX_ATTACHMENT: usize = 64 * 1024 * 1024;

/// Remove the attachment `file_name` and return its content, or `None` if
/// there is none.
pub fn take(doc: &mut Document, file_name: &str) -> Result<Option<Vec<u8>>> {
    let catalog = doc.catalog_mut()?;
    let Ok(names) = catalog.get_mut(b"Names").and_then(Object::as_dict_mut) else {
        return Ok(None);
    };
    let Ok(files) = names
        .get_mut(b"EmbeddedFiles")
        .and_then(Object::as_dict_mut)
    else {
        return Ok(None);
    };
    let Ok(list) = files.get_mut(b"Names").and_then(Object::as_array_mut) else {
        return Ok(None);
    };
    let Some(i) = list.chunks_exact(2).position(|pair| {
        pair[0]
            .as_str()
            .is_ok_and(|k| k.ends_with(file_name.as_bytes()))
    }) else {
        return Ok(None);
    };
    let spec = list[2 * i + 1].as_reference()?;
    list.drain(2 * i..2 * i + 2);
    if list.is_empty() {
        names.remove(b"EmbeddedFiles");
    }
    if names.is_empty() {
        catalog.remove(b"Names");
    }
    if let Ok(af) = catalog.get_mut(b"AF").and_then(Object::as_array_mut) {
        af.retain(|o| o.as_reference().ok() != Some(spec));
        if af.is_empty() {
            catalog.remove(b"AF");
        }
    }

    let streams: Vec<_> = doc
        .get_dictionary(spec)?
        .get(b"EF")
        .and_then(Object::as_dict)
        .context("attachment without /EF")?
        .iter()
        .filter_map(|(_, v)| v.as_reference().ok())
        .collect();
    // The PDF is untrusted: bound what a small compressed stream may expand to.
    let data = doc
        .get_object(*streams.first().context("attachment without stream")?)?
        .as_stream()?
        .get_plain_content_with_limit(MAX_ATTACHMENT)
        .context("attachment is corrupt or larger than 64 MiB")?;
    for id in streams.into_iter().chain([spec]) {
        doc.objects.remove(&id);
    }
    Ok(Some(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::dictionary;

    fn doc_with_names(names: Vec<Object>) -> Document {
        let mut doc = Document::with_version("1.7");
        let catalog = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Names" => dictionary! { "EmbeddedFiles" => dictionary! { "Names" => names } },
        });
        doc.trailer.set("Root", catalog);
        doc
    }

    /// A hostile PDF with a name list of odd length used to panic.
    #[test]
    fn odd_length_name_list_is_not_an_attachment() {
        let mut doc = doc_with_names(vec![Object::string_literal("x/typst-form.json")]);
        assert!(take(&mut doc, "typst-form.json").unwrap().is_none());
    }

    #[test]
    fn other_attachments_are_left_alone() {
        let names = vec![Object::string_literal("other.txt"), Object::Null];
        let mut doc = doc_with_names(names);
        assert!(take(&mut doc, "typst-form.json").unwrap().is_none());
    }
}
