// Regression test: a PDF/UA-1 document with form fields.
// Compile with: typst compile --root ../.. --pdf-standard ua-1 accessible.typ
#import "../../typst-package/src/lib.typ": *

#set document(title: [Accessible form])
#set text(lang: "en")

= Contact

Name: #text-field("name", tooltip: "Your name")

#check-box("news", tooltip: "Subscribe to the newsletter", label: [Newsletter])

#radio-group("size", ("S", "M", "L"), tooltip: "Size", dir: ltr, spacing: 1em)

#combo-box("country", ("Austria", "Germany"), tooltip: "Country")

#signature-field("sig", tooltip: "Signature")
