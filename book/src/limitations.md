# Limitations

- **Fonts**: field text does not follow the document font automatically;
  register it with [`form-font`](fonts.md) (TrueType outlines, Latin-1 characters).
- **Frames are page content**: hiding a field with JavaScript hides its value,
  not the frame Typst drew. Use `stroke: none, fill: none` for fields you show
  and hide.
- **PDF/A**: the field data travels as an embedded file, which PDF/A-1, -2 and
  -4 forbid: Typst stops with "document contains an attached file" (PDF/A-3 and
  PDF/UA-1 work; JavaScript is not allowed in any PDF/A).
- **Reading order**: in the structure tree, a page's fields follow that page's
  text.
