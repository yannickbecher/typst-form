# Security Policy

## Supported versions

Only the latest release receives security fixes.

## Reporting a vulnerability

Please do not open a public issue. Report it privately with GitHub's security
advisories:
<https://github.com/yannickbecher/typst-form/security/advisories/new>

Include the version (`typst-form --version`), your operating system and, if you
can, a minimal input (a PDF or `.typ` file) that triggers the problem.

This is a one-person project in spare time. I aim to answer within 7 days and
to have a fix or a clear assessment within 90 days. You are credited in the
advisory unless you prefer not to be.

## Scope

`typst-form` reads a PDF, rewrites it in place and, with `--view`, can open the
result in your PDF viewer. In scope:

- crashes, hangs, or unbounded memory use when processing a crafted or
  malformed PDF (the PDF parser is [`lopdf`](https://github.com/J-F-Liu/lopdf);
  please report parser bugs there as well),
- writing or overwriting files other than the PDF you passed, `-o` target or
  the Typst package directory of `typst-form install`,
- starting programs other than your default PDF viewer,
- tampering with release binaries or the release pipeline.

Not in scope:

- JavaScript, URLs and submit targets that the form author wrote on purpose
  (`actions`, `document-script`, `action.url`, `action.submit`): the tool puts
  them into the PDF as given, and what a PDF viewer does with them is up to the
  viewer. Only open forms from authors you trust,
- vulnerabilities in PDF viewers, Typst or other software.
