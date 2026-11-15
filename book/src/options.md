# Options for every field

| Option | Default | Meaning |
|---|---|---|
| `required` | `false` | must be filled before submitting |
| `read-only` | `false` | cannot be changed |
| `no-export` | `false` | not included when submitting |
| `hidden` | `false` | not shown |
| `tooltip` | `none` | hover text; also the accessible name |
| `tab-order` | `none` | integer, lower comes first; others follow in document order |
| `font` | `"sans"` | `"sans"`, `"serif"`, `"mono"` or a [registered font](fonts.md) |
| `font-size`, `text-color` | `auto` | the surrounding text |
| `stroke` | `0.5pt + luma(120)` | frame, drawn by Typst |
| `fill` | `white` | background, drawn by Typst |
| `radius` | `0pt` | corner radius of the frame |
| `signer` | `none` | [Acrobat Sign](people.md) role |
| `actions` | `(:)` | [JavaScript](javascript.md) per event |

`tooltip` is a string, `font-size` is `auto` or a length, `tab-order` an integer
and `text-color` a color. `width` and `height` are lengths or ratios (not `fr` or
`auto`). Buttons default to `fill: luma(235)` and `radius: 2pt`; signature
fields to a dashed grey `stroke` and a pale yellow `fill`.
