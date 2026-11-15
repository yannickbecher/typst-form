// A feedback survey: radio buttons placed freely in a table, a radio group,
// a dropdown and a free-text answer.
//
//   typst compile --root . examples/survey.typ --open typst-form
//
// Add `--input demo=true` to prefill sample answers (used for the screenshots).

#import "../typst-package/src/lib.typ": *

#let ink = rgb("#2b2140")
#let accent = rgb("#7c3aed")
#let muted = rgb("#6b6180")
#let edge = rgb("#d5cdea")
#let paper = rgb("#f8f6fd")

#let demo = sys.inputs.at("demo", default: "") == "true"
#let sample(value) = if demo { value } else { "" }

#set page(
  paper: "a4",
  margin: (x: 20mm, top: 15mm, bottom: 14mm),
  background: place(top, rect(width: 100%, height: 40mm, fill: accent)),
)
// Arial when installed, otherwise Typst's built-in Libertinus Serif.
#set text(font: ("Arial", "Libertinus Serif"), size: 9.5pt, fill: ink)
#set par(leading: 0.6em)

#let look = (fill: paper, stroke: 0.7pt + edge, radius: 3pt)

#let section(title) = block(above: 18pt, below: 9pt, width: 100%)[
  #text(size: 11pt, weight: "bold")[#title]
  #h(8pt)
  #box(width: 1fr, baseline: -3pt, line(length: 100%, stroke: 0.7pt + edge))
]

// ---------------------------------------------------------------- header

#text(size: 26pt, weight: "bold", fill: white)[How are we doing?]
#v(-4pt)
#text(size: 11pt, fill: rgb("#e4d9ff"))[Four quick questions, about one minute.]
#v(14mm)

// ----------------------------------------------------------------- scale

#section[Rate your experience]

#let questions = (
  ("Easy to find", 4),
  ("Clear to fill in", 5),
  ("Looks professional", 4),
  ("I would use it again", 5),
)

#table(
  columns: (1fr, ..(34pt,) * 5),
  align: (left + horizon, ..(center + horizon,) * 5),
  stroke: none,
  inset: (y: 7pt, x: 4pt),
  fill: (_, y) => if y > 0 and calc.odd(y) { paper },
  table.header(
    text(size: 7pt, weight: "bold", fill: muted, tracking: 0.08em)[1 = POOR · 5 = EXCELLENT],
    ..range(1, 6).map(n => text(weight: "bold", fill: accent)[#n]),
  ),
  ..questions.enumerate().map(((i, (label, pick))) => (
    [#label],
    ..range(1, 6).map(n => radio-button(
      "rating" + str(i + 1),
      str(n),
      size: 13pt,
      checked: demo and n == pick,
      ..look,
    )),
  )).flatten(),
)

// ------------------------------------------------------------------ more

#section[And one more thing]

#text(size: 7pt, weight: "bold", fill: muted, tracking: 0.08em)[WOULD YOU RECOMMEND US?]
#v(2pt)
#radio-group(
  "recommend",
  (("yes", "Yes, definitely"), ("maybe", "Maybe"), ("no", "Not yet")),
  value: if demo { "yes" } else { none },
  dir: ltr,
  spacing: 22pt,
  size: 12pt,
  ..look,
)

#v(12pt)
#grid(
  columns: (1fr, 1fr),
  column-gutter: 14pt,
  [
    #text(size: 7pt, weight: "bold", fill: muted, tracking: 0.08em)[HOW DID YOU HEAR ABOUT US?]
    #v(2pt)
    #combo-box("source", ("Search engine", "Friend or colleague", "Social media", "Conference", "Other"),
      width: 100%, value: sample("Conference"), tooltip: "How did you hear about us?", ..look)
  ],
  [
    #text(size: 7pt, weight: "bold", fill: muted, tracking: 0.08em)[EMAIL (OPTIONAL)]
    #v(2pt)
    #text-field("email", width: 100%, value: sample("ada@analytical.engine"), tooltip: "Email address", ..look)
  ],
)

#v(12pt)
#text(size: 7pt, weight: "bold", fill: muted, tracking: 0.08em)[WHAT WOULD MAKE IT BETTER?]
#v(2pt)
#text-field(
  "comments",
  width: 100%,
  height: 70pt,
  multiline: true,
  value: sample("More examples, please. The invoice one was exactly what I needed."),
  tooltip: "Comments",
  ..look,
)

#v(10pt)
#check-box("contact", size: 11pt, checked: demo, label: [You may contact me about my answers], ..look)

#v(12pt)
#submit-button(
  "send",
  "https://example.com/survey",
  label: text(fill: white, weight: "bold")[Send feedback],
  width: 120pt,
  height: 26pt,
  fill: accent,
  stroke: none,
  radius: 4pt,
)
