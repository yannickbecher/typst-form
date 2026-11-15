# Contributing to typst-form

Thank you for considering contributing to typst-form. This document explains the process and conventions.

## Getting started

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Typst](https://typst.app) CLI
- A PDF viewer for testing (Adobe Acrobat Reader recommended for JavaScript support)

### Setup

```bash
git clone https://github.com/yannickbecher/typst-form.git
cd typst-form

# Build the CLI
cd cli && cargo build --release

# Run tests
cargo test

# Test end-to-end
cd ..
typst compile --root . examples/minimal.typ test.pdf --open ./cli/target/release/typst-form
```

## Branching strategy

We use a simple trunk-based model:

| Branch | Purpose |
|--------|---------|
| `main` | Stable, release-ready code. All PRs merge here. Releases are made from it by the release workflow (below). |
| `feat/<name>` | Feature branches. Branch from `main`, PR back to `main`. |
| `fix/<name>` | Bug fix branches. Branch from `main`, PR back to `main`. |

### Branch naming

```
feat/add-barcode-field
fix/checkbox-appearance-chrome
```

### Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes
4. Run the checks: `just check` (formatting, clippy, tests)
5. Build and test with a real PDF: `just example all-fields` (or `just examples`
   for all of them). The fillable PDFs in `examples/` are committed, so
   regenerate them when you change the package, the tool or an example.
6. Open a Pull Request against `main`

## Making changes

### Adding a field option or type

1. **Typst** -- add the parameter to the function in `typst-package/src/fields.typ`
   (field-specific) and pass it in `props`, or, for an option every field has,
   add it to `defaults` in `core.typ` **and** to the metadata that `field()` emits
   (it lists the options explicitly).
2. **Rust model** -- add the matching kebab-case field to `Widget` in
   `cli/src/model.rs` with a serde default.
3. **PDF** -- set the entry in `field_dict` (field level) or `widget_dict`
   (widget level) in `cli/src/form.rs`; appearance changes go to
   `cli/src/appearance.rs`.
4. **Example** -- use it in `examples/all-fields.typ`.

### Fixing an appearance issue

Appearance streams are built in `cli/src/appearance.rs` from PDF content
stream operators (ISO 32000-1, sections 8-9). Test in several viewers:
Acrobat, Chrome, Firefox, Foxit.

### Modifying the Typst API

Update the documentation (`book/src`, an [mdBook](https://rust-lang.github.io/mdBook/)
published on GitHub Pages; preview it with `just docs`), `CHANGELOG.md` and the
examples together with the code. The README only holds the quickstart.

## Code style

### Typst

- Functions use kebab-case: `text-field`, `combo-box`
- Internal functions prefixed with underscore: `_emit`
- Comments for non-obvious logic only

### Rust

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Use `anyhow` for error handling
- PDF field flag constants: comment with the spec bit number

## Testing

### Unit tests

```bash
cd cli && cargo test
```

### End-to-end tests

```bash
# Build all examples
for f in examples/*.typ cli/tests/*.typ; do
  typst compile --root . "$f" "test_$(basename $f .typ).pdf"
  ./cli/target/release/typst-form "test_$(basename $f .typ).pdf"
done

# Open in PDF viewer and verify:
# - Fields are positioned correctly
# - Fields are interactive (clickable, fillable)
# - Checkboxes toggle
# - Radio buttons are mutually exclusive
# - Combo/list boxes show options
# - Buttons trigger actions (Acrobat only for JS)
```

### Cross-viewer testing

Test generated PDFs in:
- Adobe Acrobat Reader (reference, full JS support)
- Chrome / Edge (built-in viewer)
- Firefox (built-in viewer)
- Foxit Reader / PDF-XChange

## Commit messages

Use conventional-ish commits:

```
feat: add barcode field type
fix: checkbox not rendering in Chrome
docs: explain combo-box options
refactor: extract common field builder logic
test: add integration test for radio groups
ci: add macOS ARM64 build target
```

## Pull requests

- Keep PRs focused -- one feature or fix per PR
- Include a description of what changed and why
- Add/update examples if the change affects the user-facing API
- Ensure `cargo test` passes and `cargo clippy` is clean
- Mention which PDF viewers you tested with

## Releases

Releases use calendar versioning, `yy.nn.ff`:

| Part | Meaning | Example |
|---|---|---|
| `yy` | year | `26` |
| `nn` | release of that year, counting from 1; anything with new features or breaking changes | `26.2.0` |
| `ff` | fix release | `26.2.1` |

A new year starts at `yy.1.0`. There is no "minor or patch" decision: if it
only fixes bugs, bump `ff`; otherwise bump `nn`. No leading zeros (Cargo and
Typst require semver).

**Development vs. release.** `main` always carries the version of the last
release; only the release commit changes it. `cli/build.rs` marks builds:
`typst-form --version` prints `26.2.0` only for a clean build of the commit
tagged `v26.2.0`, and `26.2.0-dev+abc1234` (`.dirty` with uncommitted changes)
for everything else. Typst package versions cannot carry `-dev`, which is why
it is not in `typst.toml`.

### Making a release

Releases are made on GitHub, so nobody has to type a version or the year. While
you work, write what changed under `## [Unreleased]` in `CHANGELOG.md`, in the
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) style (`### Added`,
`### Changed`, `### Deprecated`, `### Removed`, `### Fixed`, `### Security`). Then start
the **Release** workflow (Actions tab, "Run workflow" on `main`) or run
`just release feature` / `just release fix` (needs the GitHub CLI `gh`):

| kind | next version | use it for |
|---|---|---|
| `feature` | `26.1.0` -> `26.2.0` | new features or breaking changes |
| `fix` | `26.2.0` -> `26.2.1` | bug fixes only |

The workflow takes the latest `v*` tag and today's date: in a new year either
kind gives `yy.1.0`; without any tag yet the first release is the version that
is in the files. Tick "dry-run" (`just release feature true`) to see the result
without pushing anything. It then

1. sets the version in `cli/Cargo.toml`, `typst-package/typst.toml` and every
   `@local/typst-form:<version>` snippet,
2. turns `## [Unreleased]` in `CHANGELOG.md` into `## [<version>] - <date>`
   (opens a new empty one and moves the compare links at the bottom); an empty
   section stops the release,
3. runs `cargo test`, which also checks that all of this agrees,
4. commits "Release <version>" to `main` and tags `v<version>`
   (`main` must allow the `github-actions` bot to push),
5. builds binaries for Linux, macOS (x64 + ARM64) and Windows from the tag and
   publishes the GitHub Release with the changelog entry as release notes,
6. publishes the documentation from the tag to GitHub Pages (once, set
   Settings > Pages > Source to "GitHub Actions").

`just check` runs formatting, clippy and the tests locally.

(`release-plz` and `cargo-release` do not fit: they derive semver versions and
know nothing about `yy.nn.ff` or `typst.toml`.)
## Security

Please report vulnerabilities privately, see [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contributions will be dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), without any additional terms or conditions.
