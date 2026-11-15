// Public field functions. All of them accept the options listed in
// `defaults` (core.typ) as named arguments.

#import "core.typ": field
#import "action.typ"

#let _styles = ("check", "circle", "cross", "diamond", "square", "star")

// `labels: "string"` (combo and list boxes: the label goes into the PDF as
// text) or `"content"` (radio groups: the label is drawn by Typst).
#let _choices(options, labels: "string") = {
  assert(type(options) == array and options.len() > 0, message: "options must be a non-empty array")
  options.map(o => {
    let (value, label) = if type(o) == array { (o.at(0), o.at(1)) } else { (o, o) }
    assert(type(value) == str, message: "option values must be strings")
    if labels == "string" {
      assert(type(label) == str, message: "combo-box and list-box labels must be strings, use (value, \"label\") pairs")
    }
    (value: value, label: label)
  })
}

/// Single- or multi-line text input.
#let text-field(
  name,
  width: 200pt,
  height: 20pt,
  value: "",
  multiline: false,
  password: false,
  file-select: false,
  comb: false,
  max-len: none,
  align: left,
  spell-check: true,
  scroll: true,
  ..common,
) = {
  assert(not comb or max-len != none, message: "comb requires max-len")
  assert(max-len == none or type(max-len) == int, message: "max-len must be an integer")
  field("text", name, width, height, common, (
    value: value,
    multiline: multiline,
    password: password,
    file-select: file-select,
    comb: comb,
    max-len: max-len,
    align: if align == center { "center" } else if align == right { "right" } else { "left" },
    spell-check: spell-check,
    scroll: scroll,
  ))
}

/// Checkbox. Several checkboxes with the same name and different `export`
/// values behave like a radio group that can be unchecked.
#let check-box(name, size: 10pt, checked: false, export: "Yes", style: "cross", label: none, ..common) = {
  assert(style in _styles, message: "style must be one of " + repr(_styles))
  assert(type(export) == str, message: "export must be a string")
  let b = field("checkbox", name, size, size, common, (checked: checked, export: export, style: style))
  if label == none { b } else { [#b#h(0.4em)#label] }
}

/// A single radio button. All buttons with the same `group` form one field,
/// so they can be placed anywhere (e.g. in table cells).
#let radio-button(
  group,
  export,
  size: 10pt,
  checked: false,
  style: "circle",
  allow-off: false,
  unison: false,
  ..common,
) = {
  assert(style in _styles, message: "style must be one of " + repr(_styles))
  assert(type(export) == str, message: "export must be a string")
  let props = (export: export, checked: checked, style: style, allow-off: allow-off, unison: unison)
  field("radio", group, size, size, common, props, shape: "circle")
}

/// Radio buttons with labels, stacked. `items` are values or (value, label) pairs.
#let radio-group(name, items, value: none, dir: ttb, spacing: 0.6em, size: 10pt, ..common) = stack(
  dir: dir,
  spacing: spacing,
  .._choices(items, labels: "content").map(item => [
    #radio-button(name, item.value, size: size, checked: item.value == value, ..common)#h(0.4em)#item.label
  ]),
)

/// Drop-down list. `options` are values or (value, label) pairs.
#let combo-box(
  name,
  options,
  value: none,
  editable: false,
  sort: false,
  commit-on-change: false,
  spell-check: true,
  width: 150pt,
  height: 20pt,
  ..common,
) = field("combo", name, width, height, common, (
  options: _choices(options),
  value: value,
  editable: editable,
  sort: sort,
  commit-on-change: commit-on-change,
  spell-check: spell-check,
), body: place(right + horizon, dx: -4pt, polygon(fill: luma(100), (0pt, 0pt), (6pt, 0pt), (3pt, 4pt))))

/// Scrollable list. `value` is a value or an array of values (multi-select).
#let list-box(
  name,
  options,
  value: (),
  multi-select: false,
  sort: false,
  commit-on-change: false,
  width: 150pt,
  height: 60pt,
  ..common,
) = field("list", name, width, height, common, (
  options: _choices(options),
  value: if type(value) == array { value } else { (value,) },
  multi-select: multi-select,
  sort: sort,
  commit-on-change: commit-on-change,
))

/// Button. `action` is a JavaScript string or one of the `action.*` helpers.
#let push-button(name, label: "Button", action: none, width: 80pt, height: 20pt, ..common) = field(
  "button",
  name,
  width,
  height,
  common,
  (action: if type(action) == str { (kind: "js", script: action) } else { action }),
  style: (fill: luma(235), radius: 2pt),
  body: align(center + horizon, label),
)

/// Button that submits the form to `url` (see `action.submit`).
#let submit-button(name, url, label: "Submit", format: "html", ..common) = push-button(
  name,
  label: label,
  action: action.submit(url, format: format),
  ..common,
)

/// Button that resets the given fields (all if none are given).
#let reset-button(name, label: "Reset", fields: (), ..common) = push-button(
  name,
  label: label,
  action: action.reset(..fields),
  ..common,
)

/// Signature field. `lock`: none, "all" or an array of field names that
/// become read-only once the field is signed.
#let signature-field(name, width: 200pt, height: 50pt, lock: none, ..common) = {
  assert(lock in (none, "all") or type(lock) == array, message: "lock must be none, \"all\" or an array")
  field("signature", name, width, height, common, (lock: lock), style: (
    stroke: (paint: luma(150), thickness: 0.5pt, dash: "dashed"),
    fill: rgb("#fffff0"),
  ))
}
