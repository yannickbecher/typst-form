# Own fonts

Field text uses Helvetica by default (`font: "serif"` and `"mono"` select Times
and Courier). For your own font, register a `.ttf` file once:

```typst
#form-font("inter", read("fonts/Inter-Regular.ttf", encoding: none))
#text-field("name", font: "inter")
```

The font is embedded completely, so viewers can show and edit any Latin-1 text
with it. Only fonts with TrueType outlines (`.ttf`) are supported.
