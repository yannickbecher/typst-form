# Validation, calculations and JavaScript

`actions` attaches JavaScript to events. The `af` module has ready-made helpers
(Acrobat's own functions) that you combine with `+`:

```typst
#text-field("a", actions: af.number(decimals: 2))
#text-field("b", actions: af.number(decimals: 2))
#text-field("total", read-only: true,
  actions: af.sum("a", "b") + af.number(decimals: 2, style: "1.234,56", currency: " €", prefix: false))

#text-field("birthday", actions: af.date("dd.mm.yyyy"))
#text-field("age", actions: af.range(min: 0, max: 120))
```

| Helper | For |
|---|---|
| `af.number(decimals: 2, style: "1,234.56", currency: "", prefix: true)` | numbers and currency |
| `af.percent(decimals: 0, style: "1,234.56")` | percentages |
| `af.date(format)`, `af.time(format)` | `"dd.mm.yyyy"`, `"HH:MM"` |
| `af.special(kind)` | `"zip"`, `"zip+4"`, `"phone"`, `"ssn"` |
| `af.mask(pattern)` | `9` digit, `A` letter, `O` either, `X` any |
| `af.range(min: none, max: none)` | allowed values |
| `af.sum(..)`, `af.product(..)`, `af.average(..)`, `af.minimum(..)`, `af.maximum(..)` | calculated fields |

When you combine helpers with `+`, the right-hand one wins for an event both
define: `af.number() + af.mask("99")` loses the number keystroke.

`style` is one of `"1,234.56"`, `"1234.56"`, `"1.234,56"`, `"1234,56"`.
Calculated fields are filled when the document opens.

## Your own scripts

Give `actions` a dictionary with these keys:

| Key | When |
|---|---|
| `keystroke` | a key is typed, before the value changes |
| `format` | the value is displayed |
| `validate` | the value has changed |
| `calculate` | another field changed |
| `focus`, `blur` | the field gains or loses focus |
| `mouse-enter`, `mouse-exit`, `mouse-down`, `mouse-up` | mouse events |

```typst
#text-field("name", actions: (validate: "if (event.value == '') app.alert('Name?');"))
```

Scripts can read and change every field: `this.getField("x").value = ..`,
`display.hidden`, `readonly`, and so on.

`document-script(name, code)` defines functions shared by all fields; it runs
when the document opens, in the order of the names.

JavaScript only runs in some viewers, see [Viewers](viewers.md).
