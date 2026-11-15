# Text

```typst
#text-field("email", width: 250pt, tooltip: "Your email address")
#text-field("notes", multiline: true, height: 60pt)
#text-field("zip", width: 70pt, comb: true, max-len: 5)
```

| Option | Default | |
|---|---|---|
| `width`, `height` | `200pt`, `20pt` | lengths or ratios (`100%`) |
| `value` | `""` | initial text |
| `multiline`, `password`, `file-select` | `false` | |
| `comb` | `false` | one cell per character, needs `max-len` |
| `max-len` | `none` | |
| `align` | `left` | `center`, `right` (anything else becomes `left`) |
| `spell-check`, `scroll` | `true` | |
