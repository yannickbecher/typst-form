# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Calendar Versioning](https://calver.org/) in the
form `yy.nn.ff`: year, release of that year, fix.

## [Unreleased]

## [26.1.0] - 2026-11-15

First release with calendar versioning. This is a rewrite of the earlier `0.x`
prototype: one `typst compile` produces everything; the tool only finishes
the PDF.

### Added

- Field data is embedded in the PDF; `typst compile form.typ --open typst-form`
  is the whole workflow. `typst-form watch` for `typst watch`.
- `typst-form install` installs the bundled Typst package as `@local/typst-form`.
- `radio-button` to place radio buttons anywhere (e.g. in tables).
- Checkbox export values and checkbox groups, `allow-off` and `unison` for radios.
- `file-select` text fields, field hierarchy via dotted names, mirrored fields
  (same name in several places), `tab-order`.
- Button actions: open URL, show/hide fields, print; submit as XFDF or PDF (`action.*`).
- Acrobat format/validate/calculate helpers: number, percent, date, time,
  special, mask, range, sum/product/average/minimum/maximum (`af.*`).
- Document-level JavaScript (`document-script`).
- Signature field locks (`lock`) and Acrobat Sign roles (`signer`).
- `--input form-debug=true` prints field names in the document.
- `form-font` embeds a TrueType font for field text.
- Tagged PDFs: every widget gets a `/Form` structure element (PDF/UA),
  pages with fields get `/Tabs /S`.
- Calculated fields are filled when the document opens.
- Font choice (`font`), font size and color default to the surrounding text.
- A checkbox shows a cross by default (`style: "check"` for a check mark).

### Changed

- The Typst package is called `typst-form` (was `acroform`): `#import "@local/typst-form:26.1.0": *`.
- `default:` is now `value:` (text, combo, list) or `checked:` (checkbox, radio).
- The `on-*` parameters are replaced by `actions: (focus: .., format: .., ..)`.
- Styling: `border:`/`fill-color:` are now Typst's `stroke:`/`fill:` (plus `radius:`).
  Typst draws the frames; the PDF widgets are transparent.
- `radio-group(.., direction: ..)` is now `dir:` (a Typst direction).
- `submit-button(name, url)` takes the URL as second argument; default format is HTML.
- `typst-form build` and `typst-form inject` are gone.

### Removed

- `placeholder` (it overlapped the typed value), `label-side`, and the
  signature options `reason`, `location`, `contact-info` (they had no effect).

### Fixed

- Non-ASCII text (umlauts) in names, values, options and tooltips.
- Submitting as PDF set the wrong flag.
- Radio groups always had "no toggle to off" set.

[Unreleased]: https://github.com/yannickbecher/typst-form/compare/v26.1.0...HEAD
[26.1.0]: https://github.com/yannickbecher/typst-form/releases/tag/v26.1.0
