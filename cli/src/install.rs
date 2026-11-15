//! `typst-form install`: copy the bundled Typst package into the local
//! package directory, so `#import "@local/typst-form:<version>": *` works.

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

const MANIFEST: &str = include_str!("../../typst-package/typst.toml");

const FILES: [(&str, &str); 9] = [
    ("typst.toml", MANIFEST),
    ("README.md", include_str!("../../typst-package/README.md")),
    ("LICENSE-MIT", include_str!("../../LICENSE-MIT")),
    ("LICENSE-APACHE", include_str!("../../LICENSE-APACHE")),
    (
        "src/lib.typ",
        include_str!("../../typst-package/src/lib.typ"),
    ),
    (
        "src/core.typ",
        include_str!("../../typst-package/src/core.typ"),
    ),
    (
        "src/fields.typ",
        include_str!("../../typst-package/src/fields.typ"),
    ),
    (
        "src/action.typ",
        include_str!("../../typst-package/src/action.typ"),
    ),
    ("src/af.typ", include_str!("../../typst-package/src/af.typ")),
];

/// Version of the bundled package (from its `typst.toml`).
pub fn version() -> &'static str {
    MANIFEST
        .lines()
        .find_map(|l| l.strip_prefix("version = \"")?.strip_suffix('"'))
        .expect("typst.toml has a version")
}

/// Install the package and return the directory it was written to.
pub fn install() -> Result<PathBuf> {
    let dir = packages_dir()?.join("local/typst-form").join(version());
    for (path, content) in FILES {
        let target = dir.join(path);
        fs::create_dir_all(target.parent().expect("has parent"))?;
        fs::write(&target, content)
            .with_context(|| format!("failed to write {}", target.display()))?;
    }
    Ok(dir)
}

/// Typst's local package directory (see `typst --help`, `--package-path`).
fn packages_dir() -> Result<PathBuf> {
    // An empty variable would make the install land in the current directory.
    let var = |name| {
        env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if let Some(path) = var("TYPST_PACKAGE_PATH") {
        return Ok(path);
    }
    let data = if cfg!(windows) {
        var("APPDATA")
    } else if cfg!(target_os = "macos") {
        var("HOME").map(|h| h.join("Library/Application Support"))
    } else {
        var("XDG_DATA_HOME").or_else(|| var("HOME").map(|h| h.join(".local/share")))
    };
    Ok(data
        .context("cannot find the data directory; set TYPST_PACKAGE_PATH")?
        .join("typst/packages"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tool installs the package it was released with.
    #[test]
    fn tool_and_package_versions_match() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }

    /// `--version` is the plain version for releases and says `-dev` otherwise.
    #[test]
    fn shown_version_marks_development_builds() {
        let shown = env!("TYPST_FORM_VERSION");
        assert!(shown.starts_with(env!("CARGO_PKG_VERSION")), "{shown}");
        let suffix = &shown[env!("CARGO_PKG_VERSION").len()..];
        assert!(suffix.is_empty() || suffix.starts_with("-dev+"), "{shown}");
    }

    /// Calendar versioning `yy.nn.ff`: year, release in that year, fix.
    #[test]
    fn version_is_calver() {
        let parts: Vec<u32> = version()
            .split('.')
            .map(|p| p.parse().expect("numeric version part"))
            .collect();
        assert_eq!(parts.len(), 3, "expected yy.nn.ff, got {}", version());
        assert!(parts[0] >= 26, "yy is the two-digit year");
        assert!(parts[1] >= 1, "nn counts releases of the year from 1");
    }

    /// Every `@local/typst-form:<version>` users can copy, in the README, the
    /// book, the package and the issue templates, names the current version.
    #[test]
    fn docs_import_the_current_version() {
        use std::path::Path;

        fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).expect("directory exists").flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                } else {
                    out.push(path);
                }
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut files = vec![root.join("README.md")];
        for dir in ["typst-package", "book/src", ".github/ISSUE_TEMPLATE"] {
            walk(&root.join(dir), &mut files);
        }

        let mut snippets = 0;
        for file in files {
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            for (at, found) in text.match_indices("@local/typst-form:") {
                let rest = &text[at + found.len()..];
                let end = rest
                    .find(|c: char| !c.is_ascii_digit() && c != '.')
                    .unwrap_or(rest.len());
                assert_eq!(
                    rest[..end].trim_end_matches('.'),
                    version(),
                    "{} names an old version",
                    file.display()
                );
                snippets += 1;
            }
        }
        assert!(snippets >= 4, "found only {snippets} import snippets");
    }

    /// The newest released changelog entry is the current version.
    #[test]
    fn changelog_starts_with_the_current_version() {
        // Headings look like `## [26.1.0] - 2026-11-15` (Keep a Changelog).
        let first = include_str!("../../CHANGELOG.md")
            .lines()
            .filter_map(|l| l.strip_prefix("## ["))
            .filter_map(|h| h.split_once(']').map(|(v, _)| v))
            .find(|v| !v.eq_ignore_ascii_case("unreleased"))
            .expect("CHANGELOG.md has a release heading");
        assert_eq!(
            first,
            version(),
            "newest changelog entry is {first}, version is {}",
            version()
        );
    }
}
