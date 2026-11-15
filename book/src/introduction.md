# typst-form

**Fillable PDF forms with Typst.** Text fields, checkboxes, radio buttons,
dropdowns, lists, buttons and signature fields -- with validation, calculations
and JavaScript, all written in your `.typ` file.

```typst
#import "@local/typst-form:26.1.0": *

Name: #text-field("name", required: true)
#check-box("agree", label: [I agree to the terms])
#submit-button("send", "https://example.com/form")
```

```bash
typst compile form.typ --open typst-form
```

`form.pdf` is now a fillable form. Start with [Getting started](getting-started.md).

## Why a second tool?

Typst cannot create form fields, and packages cannot change the PDF. So the
work is split:

- The **Typst package** draws each field's frame as normal page content and
  attaches a description of all fields to the PDF.
- The **`typst-form` tool** turns that description into real form fields in a
  few milliseconds.

A PDF that never went through the tool is a normal PDF that just shows the
frames.

## More

- Complete forms to copy from: [`examples/`](https://github.com/yannickbecher/typst-form/tree/main/examples)
- Source, issues and releases: [github.com/yannickbecher/typst-form](https://github.com/yannickbecher/typst-form)
