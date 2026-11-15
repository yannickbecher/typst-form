// An invoice that calculates itself: amounts, subtotal, VAT and total are
// computed by the PDF viewer while you type (Acrobat, Foxit, ...).
//
//   typst compile --root . examples/invoice.typ --open typst-form
//
// Add `--input demo=true` to prefill sample data (used for the screenshots).

#import "../typst-package/src/lib.typ": *

#let ink = rgb("#16302b")
#let accent = rgb("#0f766e")
#let muted = rgb("#5f7470")
#let edge = rgb("#c5d5d1")
#let paper = rgb("#f5f9f8")

#let demo = sys.inputs.at("demo", default: "") == "true"
#let sample(value) = if demo { value } else { "" }

// 1250 -> "1250.00" (the plain value of a number field).
#let money(x) = {
  let parts = str(calc.round(x, digits: 2)).split(".")
  let cents = if parts.len() == 1 { "00" } else { parts.at(1) }
  parts.at(0) + "." + cents + if cents.len() == 1 { "0" }
}

#set page(paper: "a4", margin: (x: 20mm, top: 18mm, bottom: 16mm))
// Arial when installed, otherwise Typst's built-in Libertinus Serif.
#set text(font: ("Arial", "Libertinus Serif"), size: 9.5pt, fill: ink)
#set par(leading: 0.6em)

#let look = (fill: paper, stroke: 0.7pt + edge, radius: 3pt)
#let caption(body) = text(size: 7pt, weight: "bold", fill: muted, tracking: 0.08em)[#upper(body)]

// ---------------------------------------------------------------- header

#grid(
  columns: (1fr, auto),
  align: (left + top, right + top),
  [
    #text(size: 30pt, weight: "bold", fill: accent, tracking: 0.04em)[INVOICE]
    #v(4pt)
    #grid(
      columns: (auto, 110pt),
      column-gutter: 10pt,
      row-gutter: 6pt,
      align: horizon,
      caption[Number], text-field("number", width: 100%, height: 20pt, value: sample("2027-0142"), tooltip: "Invoice number", ..look),
      caption[Date], text-field("date", width: 100%, height: 20pt, value: sample("12.02.2027"), actions: af.date("dd.mm.yyyy"), tooltip: "Invoice date", ..look),
      caption[Due], text-field("due", width: 100%, height: 20pt, value: sample("12.03.2027"), actions: af.date("dd.mm.yyyy"), tooltip: "Due date", ..look),
    )
  ],
  [
    #set text(size: 9pt)
    #text(weight: "bold", size: 11pt)[Typst Forum GmbH] \
    Karlsplatz 1 \
    1010 Vienna, Austria \
    #text(fill: muted)[billing\@typst-forum.example] \
    #text(fill: muted)[VAT ID ATU12345678]
  ],
)

#v(14pt)
#line(length: 100%, stroke: 1.2pt + accent)
#v(10pt)

#caption[Bill to]
#v(-2pt)
#text-field(
  "billto",
  width: 55%,
  height: 50pt,
  multiline: true,
  value: sample("Ada Lovelace\nAnalytical Engine Ltd.\n12 Babbage Street, London"),
  tooltip: "Customer name and address",
  ..look,
)

#v(14pt)

// ----------------------------------------------------------------- items

#let items = (
  ("Typesetting workshop, full day", 2, 480),
  ("Conference ticket, standard", 3, 250),
  ("Printed program booklet", 40, 3.5),
  none,
  none,
)

#table(
  columns: (1fr, 44pt, 70pt, 82pt),
  align: (left + horizon, right + horizon, right + horizon, right + horizon),
  stroke: none,
  inset: (y: 4pt, x: 3pt),
  table.header(
    ..([Description], [Qty], [Unit price], [Amount]).map(h => table.cell(
      fill: accent,
      inset: (x: 6pt, y: 6pt),
      text(fill: white, weight: "bold", size: 8pt)[#upper(h)],
    )),
  ),
  ..items.enumerate().map(((i, item)) => {
    let n = str(i + 1)
    let (desc, qty, price) = if item == none { ("", none, none) } else { item }
    (
      text-field("desc" + n, width: 100%, height: 20pt, value: sample(desc), tooltip: "Description of item " + n, ..look),
      text-field("qty" + n, width: 100%, height: 20pt, align: right, value: if demo and qty != none { str(qty) } else { "" }, actions: af.number(decimals: 0), tooltip: "Quantity of item " + n, ..look),
      text-field("price" + n, width: 100%, height: 20pt, align: right, value: if demo and price != none { money(price) } else { "" }, actions: af.number(decimals: 2), tooltip: "Unit price of item " + n, ..look),
      text-field(
        "amt" + n,
        width: 100%,
        height: 20pt,
        align: right,
        read-only: true,
        value: if demo and qty != none { money(qty * price) } else { "" },
        // Empty row stays empty, otherwise quantity times unit price.
        actions: af.number(decimals: 2) + (calculate: "var q = this.getField('qty" + n + "').value, p = this.getField('price" + n + "').value; event.value = (q === '' || p === '') ? '' : q * p;"),
        tooltip: "Amount of item " + n,
        ..look,
      ),
    )
  }).flatten(),
)

// ---------------------------------------------------------------- totals

#let subtotal = items.filter(i => i != none).map(((_, q, p)) => q * p).sum(default: 0)
#let vat = subtotal * 0.2

#v(8pt)
#align(right, grid(
  columns: (90pt, 82pt),
  column-gutter: 3pt,
  row-gutter: 6pt,
  align: (right + horizon, right + horizon),
  caption[Subtotal],
  text-field("subtotal", width: 100%, height: 20pt, align: right, read-only: true, value: sample(money(subtotal)),
    actions: af.sum("amt1", "amt2", "amt3", "amt4", "amt5") + af.number(decimals: 2), tooltip: "Subtotal", ..look),
  box(width: 100%, grid(columns: (1fr, 52pt), column-gutter: 6pt, align: (right + horizon, right + horizon),
    caption[VAT],
    combo-box("vatrate", (("0", "0 %"), ("10", "10 %"), ("20", "20 %")), width: 100%, height: 20pt, value: "20", tooltip: "VAT rate", ..look))),
  text-field("vat", width: 100%, height: 20pt, align: right, read-only: true, value: sample(money(vat)),
    actions: af.number(decimals: 2) + (calculate: "event.value = this.getField('subtotal').value * this.getField('vatrate').value / 100;"), tooltip: "VAT", ..look),
  caption[Total],
  text-field("total", width: 100%, height: 24pt, align: right, read-only: true, value: sample(money(subtotal + vat)),
    actions: af.number(decimals: 2, currency: "€ ", prefix: true) + (calculate: "event.value = this.getField('subtotal').value + this.getField('vat').value;"),
    tooltip: "Total", font-size: 11pt, fill: rgb("#e3f2ef"), stroke: 1pt + accent, radius: 3pt),
))

// ----------------------------------------------------------------- footer

#v(16pt)
#caption[Notes]
#v(-2pt)
#text-field("notes", width: 100%, height: 38pt, multiline: true,
  value: sample("Payment within 30 days to IBAN AT61 1904 3002 3457 3201. Thank you for your business!"),
  tooltip: "Notes", ..look)

#v(12pt)
#push-button("print", label: text(fill: white, weight: "bold")[Print invoice], action: action.print(),
  width: 110pt, height: 26pt, fill: accent, stroke: none, radius: 4pt)
#h(8pt)
#reset-button("clear", label: [Clear], width: 70pt, height: 26pt, ..look)
