# Command line

```
typst-form <file.pdf>...            add the form in place (what `--open typst-form` runs)
typst-form <file.pdf> -o <out.pdf>  write to another file
typst-form watch <file.pdf>         repeat whenever the PDF changes
typst-form install                  install the bundled Typst package
--view / TYPST_FORM_VIEW=1          open the result in the default viewer (any command)
```

A PDF without form data is left unchanged, so running the tool twice is harmless.

`typst-form --version` prints the release version (`26.1.0`) or, for a build of
the repository after that release, `26.1.0-dev+abc1234`. See [Versions](versions.md).
