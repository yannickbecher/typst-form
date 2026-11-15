// Regression test: every event on every field type, every button action.
// `cli/tests/events.pdf` is this file compiled with
//   typst compile --root . cli/tests/events.typ cli/tests/events.pdf
// (recompile it when the JSON schema changes). The tool's test checks that all
// of it ends up in the right /AA and /A entries.
#import "../../typst-package/src/lib.typ": *

// One marker script per event: `/*<field>:<event>*/`.
#let on-all(tag) = (
  keystroke: "/*" + tag + ":keystroke*/",
  format: "/*" + tag + ":format*/",
  validate: "/*" + tag + ":validate*/",
  calculate: "/*" + tag + ":calculate*/",
  focus: "/*" + tag + ":focus*/",
  blur: "/*" + tag + ":blur*/",
  mouse-enter: "/*" + tag + ":mouse-enter*/",
  mouse-exit: "/*" + tag + ":mouse-exit*/",
  mouse-down: "/*" + tag + ":mouse-down*/",
  mouse-up: "/*" + tag + ":mouse-up*/",
)

#text-field("text", actions: on-all("text"))
#check-box("check", actions: on-all("check"))
#radio-button("radio", "a", actions: on-all("radio"))
#combo-box("combo", ("x", "y"), actions: on-all("combo"))
#list-box("list", ("x", "y"), actions: on-all("list"))
#push-button("button", actions: on-all("button"))
#signature-field("signature", lock: ("text", "check"), actions: on-all("signature"))

#push-button("act-js", action: action.js("1 + 1;"))
#push-button("act-url", action: action.url("https://example.com/"))
#push-button("act-submit", action: action.submit("https://example.com/s", format: "xfdf", method: "get", fields: ("text",), empty: true))
#push-button("act-reset", action: action.reset("text", "check"))
#push-button("act-hide", action: action.hide("text"))
#push-button("act-show", action: action.hide("text", visible: true))
#push-button("act-print", action: action.print())
