# typst-form

Fillable PDF forms for Typst: text fields, checkboxes, radio buttons, combo and
list boxes, buttons and signature fields, with JavaScript, Acrobat-style
formatting and calculations.

```typst
#import "@local/typst-form:26.1.0": *

Name: #text-field("name", required: true)
#check-box("agree", label: [I agree])
#submit-button("send", "https://example.com/form")
```

Typst cannot create form fields itself. This package draws the fields and
embeds their description into the PDF; the companion tool **`typst-form`**
turns them into real form fields:

```bash
typst compile form.typ --open typst-form
```

Without the tool you get a normal PDF that shows the field frames, so the
package needs the separate `typst-form` tool (it does not work in the Typst web
app). Install it from the releases page, then run `typst-form install`.

Licensed under MIT or Apache-2.0, at your option; see `LICENSE-MIT` and
`LICENSE-APACHE` in the repository.

Documentation: https://github.com/yannickbecher/typst-form
