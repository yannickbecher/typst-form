# Buttons

```typst
#submit-button("send", "https://example.com/form", label: [Send])
#reset-button("clear", label: [Clear form])
#push-button("hello", label: [Hello], action: "app.alert('Hi!');")
```

- `submit-button(name, url, label: "Submit", format: "html")` -- `format` is
  `html`, `fdf`, `xfdf` or `pdf`. For `method`, `fields` or `empty` use
  `push-button(.., action: action.submit(..))`.
- `reset-button(name, label: "Reset", fields: ())` -- resets all fields, or only
  the named ones.
- `push-button(name, label: "Button", action: ..)` -- `action` is JavaScript or
  one of the actions below. Default size `80pt` x `20pt`.

## Ready-made actions

For `push-button(.., action: ..)`:

| Action | Effect |
|---|---|
| `action.url("https://..")` | open a web page |
| `action.submit(url, format: "html", method: "post", fields: (), empty: false)` | submit values |
| `action.reset(..fields)` | reset fields (all if none given) |
| `action.hide(..fields, visible: false)` | hide or show fields |
| `action.print()` | open the print dialog |
| `action.js(code)` | run JavaScript (a plain string does the same) |
