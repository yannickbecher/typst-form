# Accessibility

Typst produces tagged PDFs; `typst-form` adds a `/Form` structure element for
every field and sets the tab order to follow the structure, as PDF/UA requires.

Give every field a `tooltip` -- it is the field's accessible name. Compile with
`--pdf-standard ua-1` to let Typst check the rest of the document.
