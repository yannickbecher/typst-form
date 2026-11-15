// Every field type and the most important options. (German labels on purpose:
// they also test umlauts.)
//
// Compile with:
//   typst compile --root . examples/all-fields.typ --open typst-form

#import "../typst-package/src/lib.typ": *

#set page(width: 16cm, height: auto, margin: 1cm)
#set text(size: 9pt)

#document-script("helpers", "function greet(n) { app.alert('Hallo ' + n); }")

= Text
#text-field("person.first", value: "Ada", tooltip: "Vorname", required: true)
#text-field("person.last", value: "Lovelace", align: right, tab-order: 1)\
#text-field("notes", multiline: true, height: 40pt, value: "Zeile 1\nZeile 2 ist deutlich länger und sollte umbrechen, weil sie nicht in die Breite passt.")\
#text-field("pin", password: true, value: "1234", width: 60pt)
#text-field("plz", comb: true, max-len: 5, value: "12345", width: 70pt, actions: af.mask("99999"), font: "mono")
#text-field("tel", actions: af.special("phone"), width: 100pt)
#text-field("upload", file-select: true, width: 120pt)\
#text-field("betrag", value: "12.5", actions: af.number(style: "1.234,56", currency: " €", prefix: false), text-color: blue)
#text-field("datum", actions: af.date("dd.mm.yyyy"), width: 80pt)
#text-field("summe", read-only: true, actions: af.sum("betrag", "pin") + af.number(decimals: 0), width: 80pt)

= Checkboxen
#check-box("agb", checked: true, label: [AGB akzeptiert])
#check-box("haken", style: "check", label: [Haken])
#check-box("stern", style: "star", checked: true, label: [Stern])

// Farbe des Symbols: `text-color`. Rahmen und Hintergrund: `stroke`, `fill`.
#check-box("rot", checked: true, text-color: red, label: [rot])
#check-box("blau", checked: true, style: "check", text-color: blue, label: [blau])
#check-box("gruen", checked: true, style: "circle", text-color: rgb("#16a34a"), label: [grün])
#check-box("gelb", checked: true, fill: yellow, stroke: 1.5pt + rgb("#16a34a"), label: [Rahmen und Füllung])

// Gleicher Name, verschiedene `export`-Werte: höchstens eine Box ist angehakt.
#check-box("groesse", export: "klein", label: [klein], checked: true)
#check-box("groesse", export: "gross", label: [groß])

= Radiobuttons
#radio-group("anrede", ("Herr", "Frau", ("divers", "Divers")), value: "Frau", dir: ltr, spacing: 1em)

#table(columns: 3, [gut], [mittel], [schlecht],
  radio-button("note", "1"), radio-button("note", "2", checked: true), radio-button("note", "3", style: "star"))

= Auswahl
#combo-box("land", ("Deutschland", "Österreich", ("CH", "Schweiz")), value: "CH")
#combo-box("stadt", ("Berlin", "Wien"), editable: true, value: "Graz")\
#list-box("sprachen", ("Deutsch", "Englisch", "Französisch", "Spanisch"), value: ("Deutsch", "Spanisch"), multi-select: true)

= Aktionen
#push-button("hallo", label: [Hallo], action: "greet('Welt');")
#push-button("verstecken", label: [Notizen aus], action: action.hide("notes"), width: 90pt)
#push-button("drucken", label: [Drucken], action: action.print())
#push-button("web", label: [Website], action: action.url("https://typst.app"))
#submit-button("senden", "https://example.com/submit", format: "xfdf")
#reset-button("leeren", fields: ("notes", "pin"))
#text-field("geheim", hidden: true, value: "unsichtbar")

= Unterschrift
#text-field("kunde", signer: 1)
// `signer` changes the field name (here to kunde_es_:signer1): `lock` needs the full name.
#signature-field("sig-kunde", signer: 1, lock: ("kunde_es_:signer1", "person.first"))
#signature-field("sig-intern", lock: "all", actions: (focus: "app.alert('sign');"))
