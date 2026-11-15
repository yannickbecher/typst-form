# Versions

Releases use calendar versioning, `yy.nn.ff`: `26.1.0` is the first release of
2026, `26.2.0` the second, `26.2.1` a fix to it. Every release number covers the
tool and the Typst package together; see the [changelog](changelog.md).

`typst-form --version` tells a release from a development build: it prints
`26.2.0` for a release and `26.2.0-dev+abc1234` for a build of the repository
after that release (with `.dirty` if it has uncommitted changes). Download
releases from the [releases page](https://github.com/yannickbecher/typst-form/releases);
`cargo install --git` without a tag installs a development build.
