//! Put widgets into the structure tree of tagged PDFs (Typst tags by
//! default), as PDF/UA requires: each widget gets a /Form element.
//!
//! Typst does not tell us where in the reading order a field sits, so the
//! /Form elements of a page are placed after that page's content.

use std::collections::HashMap;

use anyhow::{Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId, dictionary};

/// Tag `widgets` (annotation, page), given in tab order. No-op for untagged PDFs.
pub fn tag(doc: &mut Document, widgets: &[(ObjectId, ObjectId)]) -> Result<()> {
    let Ok(root) = doc
        .catalog()?
        .get(b"StructTreeRoot")
        .and_then(Object::as_reference)
    else {
        return Ok(());
    };
    let container = document_element(doc, root)?;
    let page_numbers: HashMap<ObjectId, u32> =
        doc.get_pages().into_iter().map(|(n, id)| (id, n)).collect();

    // Page of each existing child, to find where a page's content ends.
    let kids = match doc.get_dictionary(container)?.get(b"K") {
        Ok(Object::Array(kids)) => kids.clone(),
        Ok(kid) => vec![kid.clone()],
        Err(_) => vec![],
    };
    let kid_pages: Vec<u32> = kids
        .iter()
        .map(|k| {
            page_of(doc, k, 0)
                .and_then(|p| page_numbers.get(&p).copied())
                .unwrap_or(0)
        })
        .collect();

    // New keys start behind the declared next key and behind every key that is
    // already in the parent tree, so they never collide with page content.
    let declared = doc
        .get_dictionary(root)?
        .get(b"ParentTreeNextKey")
        .and_then(Object::as_i64)
        .unwrap_or(0);
    let mut next_key = declared.max(existing_keys(doc, root));
    let mut nums = Vec::new();
    // Form elements to insert after the child at the given index (None: at the start).
    let mut inserts: Vec<(Option<usize>, Object)> = Vec::new();
    for &(annot, page) in widgets {
        let number = page_numbers.get(&page).copied().unwrap_or(0);
        let element = doc.add_object(dictionary! {
            "Type" => "StructElem",
            "S" => "Form",
            "P" => container,
            "Pg" => page,
            "K" => dictionary! { "Type" => "OBJR", "Obj" => annot, "Pg" => page },
        });
        doc.get_dictionary_mut(annot)?.set("StructParent", next_key);
        nums.extend([Object::Integer(next_key), element.into()]);
        next_key += 1;
        inserts.push((
            kid_pages.iter().rposition(|&p| p != 0 && p <= number),
            element.into(),
        ));
        doc.get_dictionary_mut(page)?.set("Tabs", "S");
    }

    // Merge the new elements into the children, keeping their order.
    let mut merged: Vec<Object> = inserts
        .iter()
        .filter(|(at, _)| at.is_none())
        .map(|(_, e)| e.clone())
        .collect();
    for (i, kid) in kids.into_iter().enumerate() {
        merged.push(kid);
        merged.extend(
            inserts
                .iter()
                .filter(|(at, _)| *at == Some(i))
                .map(|(_, e)| e.clone()),
        );
    }
    doc.get_dictionary_mut(container)?.set("K", merged);

    parent_tree(doc, root)?
        .get_mut(b"Nums")
        .and_then(Object::as_array_mut)
        .context("unsupported structure parent tree")?
        .extend(nums);
    doc.get_dictionary_mut(root)?
        .set("ParentTreeNextKey", next_key);
    Ok(())
}

/// The top-level /Document element, or the root if there is none.
fn document_element(doc: &Document, root: ObjectId) -> Result<ObjectId> {
    let first = match doc.get_dictionary(root)?.get(b"K") {
        Ok(Object::Array(kids)) => kids.first().cloned(),
        Ok(kid) => Some(kid.clone()),
        Err(_) => None,
    };
    let document = first.and_then(|k| k.as_reference().ok()).filter(|&id| {
        doc.get_dictionary(id).is_ok_and(|d| {
            d.get(b"S")
                .and_then(Object::as_name)
                .is_ok_and(|s| s == b"Document")
        })
    });
    Ok(document.unwrap_or(root))
}

/// Page of a structure element: its /Pg or that of its first descendant.
fn page_of(doc: &Document, element: &Object, depth: usize) -> Option<ObjectId> {
    let (_, element) = doc.dereference(element).ok()?;
    let dict = element.as_dict().ok()?;
    if let Ok(page) = dict.get(b"Pg").and_then(Object::as_reference) {
        return Some(page);
    }
    if depth > 16 {
        return None;
    }
    match dict.get(b"K").ok()? {
        Object::Array(kids) => kids.iter().find_map(|k| page_of(doc, k, depth + 1)),
        kid => page_of(doc, kid, depth + 1),
    }
}

/// One more than the highest key in the parent tree's direct `/Nums` (0 if none).
fn existing_keys(doc: &Document, root: ObjectId) -> i64 {
    let Ok(tree) = doc.get_dictionary(root).and_then(|r| r.get(b"ParentTree")) else {
        return 0;
    };
    let Ok((_, tree)) = doc.dereference(tree) else {
        return 0;
    };
    let nums = tree
        .as_dict()
        .and_then(|d| d.get(b"Nums"))
        .and_then(Object::as_array);
    nums.map_or(0, |nums| {
        nums.chunks_exact(2)
            .filter_map(|pair| pair[0].as_i64().ok())
            .max()
            .map_or(0, |max| max + 1)
    })
}

/// The structure parent tree (created if the document has none yet).
fn parent_tree(doc: &mut Document, root: ObjectId) -> Result<&mut Dictionary> {
    if doc.get_dictionary(root)?.get(b"ParentTree").is_err() {
        let tree = dictionary! { "Nums" => Vec::<Object>::new() };
        doc.get_dictionary_mut(root)?.set("ParentTree", tree);
    }
    match doc.get_dictionary(root)?.get(b"ParentTree")? {
        Object::Reference(id) => {
            let id = *id;
            Ok(doc.get_dictionary_mut(id)?)
        }
        _ => Ok(doc
            .get_dictionary_mut(root)?
            .get_mut(b"ParentTree")?
            .as_dict_mut()?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_keys_start_behind_the_existing_ones() {
        let mut doc = Document::with_version("1.7");
        let nums = vec![
            Object::Integer(0),
            Object::Null,
            Object::Integer(5),
            Object::Null,
        ];
        let tree = dictionary! { "Nums" => nums };
        let root = doc.add_object(dictionary! { "Type" => "StructTreeRoot", "ParentTree" => tree });
        assert_eq!(existing_keys(&doc, root), 6);

        let empty = doc.add_object(dictionary! { "Type" => "StructTreeRoot" });
        assert_eq!(existing_keys(&doc, empty), 0);
    }
}
