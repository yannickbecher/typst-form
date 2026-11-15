// Actions for `push-button(.., action: ..)`.

/// Run JavaScript.
#let js(script) = (kind: "js", script: script)

/// Submit field values to `url`.
/// - format: "html" (form-urlencoded), "fdf", "xfdf" or "pdf".
/// - fields: names to submit (default: all).
/// - empty: also submit fields without a value.
#let submit(url, format: "html", method: "post", fields: (), empty: false) = {
  assert(format in ("html", "fdf", "xfdf", "pdf"), message: "unknown submit format " + repr(format))
  assert(method in ("post", "get"), message: "method must be \"post\" or \"get\"")
  (kind: "submit", url: url, format: format, method: method, fields: fields, empty: empty)
}

/// Reset the given fields to their default values (all if none are given).
#let reset(..fields) = (kind: "reset", fields: fields.pos())

/// Open a URL.
#let url(target) = (kind: "url", url: target)

/// Hide the given fields (or show them with `visible: true`).
#let hide(..fields, visible: false) = (kind: "hide", fields: fields.pos(), hide: not visible)

/// Open the print dialog.
#let print() = js("this.print();")
