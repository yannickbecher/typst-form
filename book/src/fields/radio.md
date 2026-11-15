# Radio buttons

```typst
#radio-group("size", ("S", "M", "L"), value: "M", dir: ltr)
#radio-group("plan", (("free", [Free]), ("pro", [Pro])))
```

`items` are values or `(value, label)` pairs. Options: `value` (selected),
`dir: ttb`, `spacing: 0.6em`, `size: 10pt`.

To place buttons anywhere (for example in a table), use `radio-button`. All
buttons with the same group name form one field:

```typst
#radio-button("rating", "good")  #radio-button("rating", "bad")
```

`radio-button(group, export, ..)` takes `size`, `checked`, `style: "circle"`,
`allow-off: false` (clicking the selected button unselects it) and
`unison: false` (buttons with the same export value switch together).
