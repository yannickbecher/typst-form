// A conference registration: header band, soft rounded fields, radio buttons,
// checkboxes, dropdowns, a signature and styled buttons.
//
//   typst compile --root . examples/registration.typ --open typst-form
//
// Add `--input demo=true` to prefill sample data (used for the screenshots).

#import "../typst-package/src/lib.typ": *

#let ink = rgb("#1b2a41")
#let accent = rgb("#2f6fed")
#let muted = rgb("#5b6b82")
#let edge = rgb("#c3cfe2")
#let paper = rgb("#f6f8fc")

#let demo = sys.inputs.at("demo", default: "") == "true"
#let sample(value) = if demo { value } else { "" }

#set page(
  paper: "a4",
  margin: (x: 20mm, top: 15mm, bottom: 14mm),
  background: place(top, rect(width: 100%, height: 42mm, fill: ink)),
)
// Arial when installed, otherwise Typst's built-in Libertinus Serif.
#set text(font: ("Arial", "Libertinus Serif"), size: 9.5pt, fill: ink)
#set par(leading: 0.6em)

// How every field looks: soft fill, thin border, rounded corners.
#let look = (fill: paper, stroke: 0.7pt + edge, radius: 3pt)

#let labelled(label, body) = block(breakable: false, below: 10pt)[
  #text(size: 7pt, weight: "bold", fill: muted, tracking: 0.08em)[#upper(label)]
  #v(-2pt)
  #body
]

#let section(title) = block(above: 16pt, below: 9pt, width: 100%)[
  #text(size: 11pt, weight: "bold")[#title]
  #h(8pt)
  #box(width: 1fr, baseline: -3pt, line(length: 100%, stroke: 0.7pt + edge))
]

#let two(a, b) = grid(columns: (1fr, 1fr), column-gutter: 14pt, a, b)

// ---------------------------------------------------------------- header

#text(size: 26pt, weight: "bold", fill: white)[Conference Registration]
#v(-4pt)
#text(size: 11pt, fill: rgb("#a9bfe8"))[Typst Forum 2027 · Vienna · 14–16 May]
#v(13mm)

// ------------------------------------------------------------- the form

#section[Your details]

#two(
  labelled("First name", text-field("first", width: 100%, value: sample("Ada"), required: true, tooltip: "First name", ..look)),
  labelled("Last name", text-field("last", width: 100%, value: sample("Lovelace"), required: true, tooltip: "Last name", ..look)),
)
#labelled("Email", text-field("email", width: 100%, value: sample("ada@analytical.engine"), required: true, tooltip: "Email address", ..look))
#two(
  labelled("Organisation", text-field("org", width: 100%, value: sample("Analytical Engine Ltd."), tooltip: "Organisation", ..look)),
  labelled("Role", combo-box("role", ("Developer", "Designer", "Researcher", "Student", "Other"), width: 100%, value: sample("Researcher"), tooltip: "Role", ..look)),
)

#section[Ticket]

#radio-group(
  "ticket",
  (("standard", "Standard · €250"), ("student", "Student · €90"), ("speaker", "Speaker · free")),
  value: if demo { "standard" } else { none },
  dir: ltr,
  spacing: 22pt,
  size: 11pt,
  ..look,
)

#v(10pt)
#labelled("Days you attend")[
  #check-box("day1", checked: demo, size: 11pt, label: [Wed · Workshops], ..look)
  #h(16pt)
  #check-box("day2", checked: demo, size: 11pt, label: [Thu · Talks], ..look)
  #h(16pt)
  #check-box("day3", size: 11pt, label: [Fri · Hack day], ..look)
]

#two(
  labelled("T-shirt", combo-box("tshirt", ("XS", "S", "M", "L", "XL"), width: 100%, value: sample("M"), tooltip: "T-shirt size", ..look)),
  labelled("Dietary needs", combo-box("diet", ("None", "Vegetarian", "Vegan", "Gluten-free"), width: 100%, value: sample("Vegetarian"), tooltip: "Dietary needs", ..look)),
)

#section[Anything else?]

#text-field(
  "notes",
  width: 100%,
  height: 54pt,
  multiline: true,
  value: sample("I would love a seat near the stage. Looking forward to the typesetting workshop!"),
  tooltip: "Notes",
  ..look,
)

#section[Confirm]

#check-box("terms", size: 11pt, required: true, checked: demo, label: [I agree to the terms and the privacy policy], ..look)
#v(10pt)

#grid(
  columns: (1fr, 110pt),
  column-gutter: 14pt,
  labelled("Signature", signature-field("signature", width: 100%, height: 44pt, tooltip: "Signature", ..look)),
  labelled("Date", text-field("date", width: 100%, height: 22pt, value: sample("12.02.2027"), actions: af.date("dd.mm.yyyy"), tooltip: "Date", ..look)),
)

#v(8pt)
#submit-button(
  "submit",
  "https://example.com/register",
  label: text(fill: white, weight: "bold")[Submit registration],
  width: 150pt,
  height: 26pt,
  fill: accent,
  stroke: none,
  radius: 4pt,
)
#h(8pt)
#reset-button("reset", label: [Clear form], width: 90pt, height: 26pt, ..look)
