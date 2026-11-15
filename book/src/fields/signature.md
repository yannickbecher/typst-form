# Signature

```typst
#signature-field("sig", lock: "all")
#signature-field("sig2", lock: ("name", "date"))
```

`lock` makes all (or the named) fields read-only as soon as the document is
signed. This works in every standards-compliant viewer. Default size
`200pt` x `50pt`.

## Do something when it is signed

To fill in something when the document is signed, give the signature field a
`validate` script. Acrobat runs it after signing (it shows it as the field's
"Signed" script):

```typst
#text-field("signed-on", read-only: true)
#signature-field("sig", actions: (
  validate: "this.getField('signed-on').value = util.printd('yyyy-mm-dd', new Date());",
))
```

Whether a viewer runs it is up to the viewer; only Acrobat is known to.
