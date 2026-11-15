set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

tool := if os() == "windows" { ".\\cli\\target\\release\\typst-form.exe" } else { "./cli/target/release/typst-form" }

# Run `just` to list the recipes.
default:
    @just --list

# Build the typst-form tool.
build:
    cargo build --release --manifest-path cli/Cargo.toml

# Format check, lint and tests.
check:
    cargo fmt --manifest-path cli/Cargo.toml -- --check
    cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings
    cargo test --manifest-path cli/Cargo.toml

# Compile one example (examples/<name>.typ) to a fillable examples/<name>.pdf.
example name: build
    typst compile --root . examples/{{name}}.typ examples/{{name}}.pdf
    {{tool}} examples/{{name}}.pdf

# Compile all examples to fillable PDFs in examples/.
[unix]
examples: build
    for f in examples/*.typ; do just example "$(basename "$f" .typ)"; done

# Compile all examples to fillable PDFs in examples/.
[windows]
examples: build
    foreach ($f in Get-ChildItem examples/*.typ) { just example $f.BaseName; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE } }

# Preview the documentation in the browser (needs `mdbook`).
docs:
    mdbook serve book --open

# Release on GitHub (needs `gh`). kind: feature | fix. dry=true pushes nothing.
release kind dry="false":
    gh workflow run release.yml --ref main -f kind={{kind}} -f dry-run={{dry}}
