// typst-form: fillable PDF form fields for Typst.
//
//   #import "@local/typst-form:26.1.0": *
//   #text-field("name")
//
// Compile as usual and let Typst run the typst-form tool afterwards:
//   typst compile form.typ --open typst-form

#import "core.typ": document-script, form-font
#import "fields.typ": (
  check-box, combo-box, list-box, push-button, radio-button, radio-group, reset-button,
  signature-field, submit-button, text-field,
)
#import "action.typ"
#import "af.typ"
