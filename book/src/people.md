# Assigning fields to people

PDF has no per-user permissions. Two mechanisms cover the usual needs:

- **Acrobat Sign roles**: `signer: 1` (or `2`, `"prefill"`, `(1, "initials")`)
  appends Acrobat Sign's text-tag syntax to the field name, for example
  `signature_es_:signer1:signature`. When the PDF is sent through Acrobat Sign,
  the field belongs to that recipient. Other viewers just see a longer name.
  Scripts, `lock`, `fields:` and `af.*` must use that full name, for example
  `kunde_es_:signer1`.
- **Signature locks**: see [Signature](fields/signature.md).
