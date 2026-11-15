// Ready-made field actions based on Acrobat's built-in AF* functions
// (supported by Acrobat, Foxit, PDF-XChange and pdf.js). They return
// dictionaries for `actions:` and can be combined with `+`:
//
//   #text-field("total", read-only: true,
//     actions: af.sum("a", "b") + af.number(decimals: 2, currency: " €", prefix: false))

#let _q(s) = json.encode(s)
#let _sep = ("1,234.56": 0, "1234.56": 1, "1.234,56": 2, "1234,56": 3)
#let _call(f, ..args) = f + "(" + args.pos().map(a => if type(a) == str { a } else { repr(a) }).join(", ") + ");"

/// Number with fixed decimals. `style` is one of "1,234.56", "1234.56",
/// "1.234,56", "1234,56".
#let number(decimals: 2, style: "1,234.56", currency: "", prefix: true) = {
  let args = (decimals, _sep.at(style), 0, 0, _q(currency), prefix)
  (keystroke: _call("AFNumber_Keystroke", ..args), format: _call("AFNumber_Format", ..args))
}

/// Percentage (the value 0.5 is shown as 50%).
#let percent(decimals: 0, style: "1,234.56") = (
  keystroke: _call("AFPercent_Keystroke", decimals, _sep.at(style)),
  format: _call("AFPercent_Format", decimals, _sep.at(style)),
)

/// Date, e.g. format: "dd.mm.yyyy".
#let date(format) = (
  keystroke: _call("AFDate_KeystrokeEx", _q(format)),
  format: _call("AFDate_FormatEx", _q(format)),
)

/// Time, e.g. format: "HH:MM".
#let time(format) = (
  keystroke: _call("AFTime_KeystrokeEx", _q(format)),
  format: _call("AFTime_FormatEx", _q(format)),
)

/// US formats from Acrobat's "Special" category: "zip", "zip+4", "phone", "ssn".
#let special(kind) = {
  let n = ("zip": 0, "zip+4": 1, "phone": 2, "ssn": 3).at(kind)
  (keystroke: _call("AFSpecial_Keystroke", n), format: _call("AFSpecial_Format", n))
}

/// Only accept input matching a mask: 9 = digit, A = letter, O = either,
/// X = any character, e.g. "99999" or "AA-9999".
#let mask(pattern) = (keystroke: _call("AFSpecial_KeystrokeEx", _q(pattern)))

/// Reject values outside [min, max] (either bound may be none).
#let range(min: none, max: none) = (
  validate: _call("AFRange_Validate", min != none, if min == none { 0 } else { min },
    max != none, if max == none { 0 } else { max }),
)

#let _simple(op, fields) = (
  calculate: _call("AFSimple_Calculate", _q(op), "new Array(" + fields.pos().map(_q).join(", ") + ")"),
)

/// Value is the sum of the given fields.
#let sum(..fields) = _simple("SUM", fields)
/// Value is the product of the given fields.
#let product(..fields) = _simple("PRD", fields)
/// Value is the average of the given fields.
#let average(..fields) = _simple("AVG", fields)
/// Value is the minimum of the given fields.
#let minimum(..fields) = _simple("MIN", fields)
/// Value is the maximum of the given fields.
#let maximum(..fields) = _simple("MAX", fields)
