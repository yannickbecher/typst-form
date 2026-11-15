// The simplest possible form.
//
// Compile with:
//   typst compile --root . examples/minimal.typ --open typst-form

#import "../typst-package/src/lib.typ": *

= Hello Form

Name: #text-field("name")

#check-box("agree", label: [I agree])
