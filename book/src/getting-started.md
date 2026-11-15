# Getting started

`typst-form` requires [Typst](https://typst.app) 0.15 or newer.

## Installation

**1. Install the `typst-form` tool.** Download it from the
[releases](https://github.com/yannickbecher/typst-form/releases) and put it on
your `PATH`, or build it with Rust:

```bash
cargo install --git https://github.com/yannickbecher/typst-form typst-form
```

**2. Register the Typst package.** It is bundled with the tool:

```bash
typst-form install
```

The command prints the import line you need.

## Your first form

```typst
#import "@local/typst-form:26.1.0": *

= Registration

Name: #text-field("name", required: true)

Country: #combo-box("country", ("Austria", "Germany", "Switzerland"))

#check-box("agree", label: [I agree to the terms])

#submit-button("send", "https://example.com/form")
```

Compile it and let Typst run `typst-form` afterwards:

```bash
typst compile form.typ --open typst-form
```

`form.pdf` is now a fillable form.

## Working on a form

While you edit, run the tool next to `typst watch`. It adds the form every time
Typst rewrites the PDF:

```bash
typst watch form.typ
typst-form watch form.pdf
```

Add `--view` (or set `TYPST_FORM_VIEW=1`) to open the result in your PDF viewer.
Typst's own `--open` is taken by the tool.
