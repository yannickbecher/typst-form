# Troubleshooting

**The PDF has no fields.** The tool did not run. Check `typst-form --version`
and that you passed `--open typst-form`. Running `typst-form form.pdf` by hand
does the same.

**"failed to replace ... locked".** A viewer (for example Acrobat) keeps the
PDF open. Close it, or use one that reloads files (SumatraPDF, Okular, Skim,
VS Code).

**Which name does a field have?** `typst compile form.typ --input form-debug=true`
prints every field name below its frame.

**A console window flashes on Windows.** That is Typst starting the tool.
It is harmless.

**`#import "@local/typst-form:26.1.0": *` fails.** Run `typst-form install`
once, and use the import line it prints. The version of the package is the
version of the tool, so a newer tool needs the newer version in the import.
