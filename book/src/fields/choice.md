# Dropdown and list

```typst
#combo-box("country", ("Austria", "Germany"), editable: true, value: "Austria")
#list-box("hobbies", ("Music", "Sport", "Books"), multi-select: true)
```

`options` are strings or `(value, "label")` pairs; labels must be strings, not
content. Both take `value` (for a list box a string or an array), `sort` and
`commit-on-change`.

| | Extra options | Default size |
|---|---|---|
| `combo-box` | `editable`, `spell-check` | `150pt` x `20pt` |
| `list-box` | `multi-select` | `150pt` x `60pt` |
