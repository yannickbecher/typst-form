//! Tells a release from a development build.
//!
//! `TYPST_FORM_VERSION` (shown by `typst-form --version`) is the plain version
//! for a clean build of the commit tagged `v<version>`, and otherwise
//! `<version>-dev+<commit>` (plus `.dirty` with uncommitted changes). Without
//! git (a source tarball) it is the plain version.

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    let full = match git(&["rev-parse", "--short=7", "HEAD"]) {
        None => version.to_owned(),
        Some(commit) => {
            let tag = format!("v{version}");
            let tagged = git(&["describe", "--tags", "--exact-match", "--match", &tag]).is_some();
            let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
                .is_some_and(|s| !s.is_empty());
            match (tagged, dirty) {
                (true, false) => version.to_owned(),
                (_, false) => format!("{version}-dev+{commit}"),
                (_, true) => format!("{version}-dev+{commit}.dirty"),
            }
        }
    };
    println!("cargo:rustc-env=TYPST_FORM_VERSION={full}");

    // Re-run when the checked-out commit, the tags or the index change.
    for path in ["HEAD", "index", "refs/tags", "packed-refs"] {
        println!("cargo:rerun-if-changed=../.git/{path}");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
