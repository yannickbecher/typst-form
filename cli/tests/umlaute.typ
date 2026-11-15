// Regression test: non-ASCII text in names, values, tooltips and options.
#import "../../typst-package/src/lib.typ": *

Straße: #text-field("straße", value: "Müllerstraße 5", tooltip: "Straße & Hausnummer")

Land: #combo-box("land", ("Deutschland", "Österreich", "Schweiz"), value: "Österreich")

#list-box("größe", ("Klein", "Groß"), value: "Groß")

#check-box("agb", label: [Ich stimme den AGB zu], tooltip: "Bestätigung")

#radio-group("anrede", (("herr", "Herr"), ("frau", "Frau")), value: "frau", allow-off: true)

#push-button("hallo", label: [Grüß dich], action: "app.alert('Grüß Gott')")
