// Core of the package: every field records its position and properties as
// `metadata(..) <form-field>`. The first one also attaches all of them as
// `typst-form.json` to the PDF, which the `typst-form` tool turns into real
// AcroForm fields.

#let _count = counter("typst-form")

// Options every field accepts.
#let defaults = (
  required: false,
  read-only: false,
  no-export: false,
  hidden: false,
  tooltip: none,
  tab-order: none,
  // Acrobat Sign recipient: 1, 2, .. | "prefill" | (role, field-type)
  signer: none,
  // JavaScript per event, see `af.typ` for ready-made ones.
  actions: (:),
  // "sans", "serif", "mono" or a name registered with `form-font`.
  font: "sans",
  font-size: auto,
  text-color: auto,
  // Look of the field on the page (drawn by Typst, the widget is transparent).
  stroke: 0.5pt + luma(120),
  fill: white,
  radius: 0pt,
)

#let _events = (
  "keystroke", "format", "validate", "calculate",
  "focus", "blur", "mouse-enter", "mouse-exit", "mouse-down", "mouse-up",
)
#let _fonts = (sans: "Helv", serif: "TiRo", mono: "Cour")

#let _pt(v, size) = {
  if type(v) == length { v.to-absolute().pt() }
  else if type(v) == ratio { (v * size).pt() }
  else { (v.length.to-absolute() + v.ratio * size).pt() }
}

#let _rgb(c) = rgb(c).components(alpha: false).map(v => v / 100%)

// Acrobat Sign text tag syntax: `name_es_:signer1[:type]`.
#let _tagged(name, signer, kind) = {
  if signer == none { return name }
  let (role, field-type) = if type(signer) == array { signer } else {
    (signer, if kind == "signature" { "signature" })
  }
  let role = if type(role) == int { "signer" + str(role) } else { role }
  name + "_es_:" + role + if field-type != none { ":" + field-type } else { "" }
}

#let _attach() = context pdf.attach(
  "/typst-form.json",
  bytes(json.encode(query(<form-field>).map(m => m.value), pretty: false)),
  relationship: "data",
  mime-type: "application/json",
  description: "Form field definitions, removed by the typst-form tool",
)

// Record `data` (plus position and size if `frame` is given) for the tool.
// Entries without frame are not boxed, so they take no space in the text.
#let _emit(data, frame: none, width: 0pt, height: 0pt) = (if frame == none { x => x } else { box })({
  _count.step()
  context {
    if frame == none {
      [#metadata(data) <form-field>]
    } else {
      let pos = here().position()
      layout(size => [#frame#metadata((
        ..data,
        page: pos.page,
        x: pos.x.pt(),
        y: pos.y.pt(),
        width: _pt(width, size.width),
        height: _pt(height, size.height),
      )) <form-field>])
    }
    if _count.get() == (1,) { _attach() }
  }
})

/// Shared implementation of all field functions.
///
/// - kind: field type understood by the tool.
/// - args: the caller's `..common` arguments (validated against `defaults`).
/// - props: type-specific properties.
/// - style: overrides of `defaults` for this field type.
/// - body: content drawn inside the frame.
#let field(kind, name, width, height, args, props, style: (:), shape: "rect", body: none) = {
  assert(type(name) == str and name != "", message: "field name must be a non-empty string")
  assert(args.pos() == (), message: "unexpected positional argument " + repr(args.pos()))
  let o = defaults + style
  for (key, value) in args.named() {
    assert(key in o, message: "unknown argument `" + key + "`, expected one of: " + o.keys().join(", "))
    o.insert(key, value)
  }
  assert(type(width) in (length, ratio, relative), message: "width must be a length or a ratio (fr and auto are not supported)")
  assert(type(height) in (length, ratio, relative), message: "height must be a length or a ratio (fr and auto are not supported)")
  assert(o.tooltip == none or type(o.tooltip) == str, message: "tooltip must be a string")
  assert(o.font-size == auto or type(o.font-size) == length, message: "font-size must be auto or a length")
  assert(o.text-color == auto or type(o.text-color) == color, message: "text-color must be auto or a color")
  for event in o.actions.keys() {
    assert(event in _events, message: "unknown event `" + event + "` in actions, expected one of: " + _events.join(", "))
  }

  let frame = box(
    width: width,
    height: height,
    stroke: o.stroke,
    fill: o.fill,
    radius: if shape == "circle" { 50% } else { o.radius },
    {
      body
      // `typst compile --input form-debug=true` labels every field with its name.
      if sys.inputs.at("form-debug", default: none) == "true" {
        place(bottom + left, dy: 0.9em, text(5pt, red, font: "DejaVu Sans Mono", fallback: true, name))
      }
    },
  )
  context _emit(
    (
      type: kind,
      name: _tagged(name, o.signer, kind),
      required: o.required,
      read-only: o.read-only,
      no-export: o.no-export,
      hidden: o.hidden,
      tooltip: o.tooltip,
      tab-order: o.tab-order,
      actions: o.actions,
      font: _fonts.at(o.font, default: o.font),
      font-size: if o.font-size == auto { text.size.to-absolute().pt() } else { o.font-size.to-absolute().pt() },
      text-color: _rgb(if o.text-color != auto { o.text-color }
        else if type(text.fill) == color { text.fill } else { black }),
      ..props,
    ).pairs().filter(((_, v)) => v != none).to-dict(),
    frame: if o.hidden { hide(frame) } else { frame },
    width: width,
    height: height,
  )
}

/// JavaScript that runs when the document is opened (document-level script).
/// Use it to define functions shared by several fields.
#let document-script(name, code) = _emit((type: "script", name: name, code: code))

/// Use a TrueType font (.ttf) for field text: register it once, then pass
/// `font: name` to fields. The font file is embedded in the PDF, so
/// viewers can show and edit the field text with it.
///
/// `#form-font("inter", read("fonts/Inter-Regular.ttf", encoding: none))`
#let form-font(name, data) = {
  assert(type(data) == bytes, message: "form-font expects the font file as bytes: read(\"font.ttf\", encoding: none)")
  assert(name.match(regex("^[A-Za-z][A-Za-z0-9-]*$")) != none, message: "font name must be letters, digits and dashes")
  assert(name not in _fonts, message: "`" + name + "` is a built-in font name")
  _emit((type: "font", name: name))
  pdf.attach("/typst-form-font-" + name + ".ttf", data, mime-type: "font/ttf",
    description: "Font for form fields, moved into the form by the typst-form tool")
}
