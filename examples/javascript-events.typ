// Calculations, formatting, validation and custom JavaScript.
//
// Compile with:
//   typst compile --root . examples/javascript-events.typ --open typst-form
//
// JavaScript runs in Acrobat (the reference) and, expected to, Foxit and PDF-XChange; Firefox/pdf.js runs part of it.

#import "../typst-package/src/lib.typ": *

#set page(margin: 2cm)
#set text(size: 10pt)

// Shared helper, available to every field script.
#document-script("helpers", "function product(a, b) { return this.getField(a).value * this.getField(b).value; }")

#let money = af.number(decimals: 2, currency: "$ ")
#let cell(name, ..args) = text-field(name, width: 80pt, align: right, ..args)

= Invoice

#table(
  columns: (1fr, auto, auto, auto),
  align: (left, right, right, right),
  stroke: 0.5pt + gray,
  [*Item*], [*Quantity*], [*Unit price*], [*Subtotal*],
  ..for i in (1, 2, 3) {
    let (qty, price, sub) = ("qty" + str(i), "price" + str(i), "sub" + str(i))
    (
      [Item #i],
      cell(qty, value: "1", actions: af.number(decimals: 0) + af.range(min: 0, max: 999)),
      cell(price, value: "9.99", actions: money),
      cell(sub, read-only: true, fill: luma(245),
        actions: money + (calculate: "event.value = product('" + qty + "', '" + price + "');")),
    )
  },
)

#align(right)[
  *Total* #text-field("total", width: 100pt, read-only: true, align: right, fill: luma(235),
    actions: af.sum("sub1", "sub2", "sub3") + money)
]

== Events

#text-field("focus-demo", width: 250pt, actions: (
  focus: "event.target.fillColor = color.ltGray;",
  blur: "event.target.fillColor = color.transparent;",
))

#v(6pt)
#push-button("hello", label: [Say hello], width: 100pt,
  action: "app.alert('Total: ' + this.getField('total').valueAsString);")
#push-button("hide", label: [Hide note], width: 80pt, action: action.hide("note"))
#push-button("show", label: [Show note], width: 80pt, action: action.hide("note", visible: true))

// Fields that are shown/hidden by script should not have a Typst frame,
// because only the widget itself is hidden.
#text-field("note", value: "Now you see me", read-only: true, stroke: none, fill: none)
#push-button("print", label: [Print], action: action.print())
