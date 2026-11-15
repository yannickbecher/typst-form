// Regression test: a document that contains nothing but fields (its
// structure tree has no parent tree yet).
#import "../../typst-package/src/lib.typ": *

#text-field("a", value: "1", actions: af.number(decimals: 0))
