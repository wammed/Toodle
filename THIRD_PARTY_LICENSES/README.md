# Toodle Third-Party License Audit

Last reviewed: **2026-09-21**

# Third-Party License Records

This directory is maintained separately from the main project documentation.

Do not rewrite, summarize, or restructure this document as part of routine
project documentation updates.

This document records third-party dependency license information,
license investigations, and cargo-deny findings.

This directory records the license evidence for dependencies that currently
produce `unlicensed` diagnostics in `cargo-deny 0.20.2` because their
individual Cargo package metadata does not contain a `license` or
`license-file` field.

This document is an audit record, not a replacement for the upstream license
texts. The authoritative license files remain in the dependency source trees.

## Audit policy

Toodle uses `cargo-deny` with an explicit SPDX allowlist.

The policy deliberately does **not** convert every upstream metadata deficiency
into an unconditional `allow` or crate-wide license exception. An `allow`
entry means that license is acceptable wherever it appears in the dependency
graph; it does not establish that a particular crate is licensed under that
expression.

Where upstream metadata is incomplete, the evidence is recorded here:

1. exact dependency/repository revision where known;
2. location of the authoritative license text;
3. license identified from that text and/or source headers;
4. reason `cargo-deny` still reports the crate as unlicensed.

When an upstream revision changes, this audit entry must be re-checked.

## Current cargo-deny policy

`deny.toml` allows these SPDX licenses:

- 0BSD
- Apache-2.0
- Apache-2.0 WITH LLVM-exception
- BSD-2-Clause
- BSD-3-Clause
- BSL-1.0
- CC0-1.0
- GPL-3.0-or-later
- ISC
- MIT
- MPL-2.0
- Unicode-3.0
- Unlicense
- Zlib

`GPL-3.0-or-later` is included because it is Toodle's own project license.

`CDLA-Permissive-2.0` is intentionally not in the allowlist. The `reqwest`
configuration was changed to use `rustls-tls-native-roots`, and the audited
dependency graph no longer contains `webpki-roots 1.0.9`.

## Dependencies with incomplete upstream Cargo metadata

### libcosmic family

Dependency source:

`https://github.com/pop-os/libcosmic`

Audited revision:

`87ab8179`

Evidence:

- repository root `LICENSE` is Mozilla Public License Version 2.0;
- the source tree contains many `SPDX-License-Identifier: MPL-2.0`
  headers;
- some source files explicitly use `MPL-2.0 AND MIT`;
- the workspace package metadata does not define a `license` field;
- the affected child crates also do not define `license` or `license-file`.

Affected packages observed by `cargo-deny`:

| Crate | Version | Audit result |
|---|---:|---|
| `libcosmic` | 1.0.0 | MPL-2.0 evidence; Cargo metadata missing |
| `cosmic-config` | 1.0.0 | MPL-2.0 repository evidence; Cargo metadata missing |
| `cosmic-config-derive` | 1.0.0 | MPL-2.0 repository evidence; Cargo metadata missing |
| `cosmic-theme` | 1.0.0 | MPL-2.0 repository evidence; Cargo metadata missing |

`cosmic-theme` was checked directly at the audited revision. No crate-local
LICENSE/COPYING file and no SPDX header were found under that crate. Its
classification therefore relies on the repository-level MPL-2.0 license and
the surrounding libcosmic source licensing evidence.

### cosmic-settings-daemon

Dependency source:

`dbus-settings-bindings` Git repository

Audited revision:

`eed01dd3609e90e3c8cd043656734c500956c793`

Evidence:

- repository root `LICENSE.md` begins with Mozilla Public License Version 2.0;
- `cosmic-settings-daemon/Cargo.toml` has no `license` or `license-file`;
- `cosmic-settings-daemon/` contains no crate-local LICENSE/COPYING file;
- no `SPDX-License-Identifier` header was found under that crate.

Audit classification:

**MPL-2.0, based on the repository-level license; upstream Cargo metadata is
incomplete.**

### dnd and mime

Dependency source:

`https://github.com/pop-os/window_clipboard.git?tag=sctk-0.20`

Audited revision:

`f68595ee0e62fbd6589f4709b5aaa5c3c7ea5f6c`

Evidence:

- repository root `LICENSE` is the MIT license;
- repository root `Cargo.toml` declares `license = "MIT"`;
- `dnd/Cargo.toml` has no `license` or `license-file`;
- `mime/Cargo.toml` has no `license` or `license-file`;
- neither crate has its own LICENSE file;
- the `dnd` and `mime` crates were introduced by the `228288df...`
  `feat: dnd integration` commit without adding a separate crate license
  file.

Audit classification:

**MIT, based on the repository-level MIT license and root package metadata;
crate-level Cargo metadata is incomplete.**

Because the repository-level license applies to the repository source but the
child crates do not explicitly declare it, this is recorded as an upstream
metadata deficiency rather than silently treating the crates as machine-
verified by cargo-deny.

## apply

Package:

`apply 0.3.0`

Evidence:

- Cargo metadata uses `license-file = "./License.md"`;
- `License.md` contains the Unlicense/public-domain dedication.

Audit classification:

**Unlicense.**

This is a normal crate-local license-file case and does not require a special
audit exception.

## Why these crates are not added to `exceptions`

`licenses.exceptions` is not a general-purpose "known-good crate" list. It
changes which license expressions are accepted for a particular crate.

The affected Git dependencies have a different problem: their Cargo metadata
does not expose the license expression at all. Adding broad exceptions would
make the configuration look cleaner while weakening the audit signal.

The preferred long-term fix is upstream metadata such as:

```toml
license = "MPL-2.0"
```

or:

```toml
license = "MIT"
```

in the corresponding package manifests, where that accurately reflects the
upstream licensing policy.

## Why repository-level LICENSE files are not used as cargo-deny clarifications

The affected Git crates use repository-level license files outside their
individual crate roots, for example:

```text
dbus-settings-bindings/
  LICENSE.md
  cosmic-settings-daemon/
    Cargo.toml
```

A repository-level license therefore cannot safely be represented as a
crate-local `license-files` clarification such as:

```toml
license-files = [{ path = "../LICENSE.md", ... }]
```

The same structural issue applies to the libcosmic workspace and to
`window_clipboard`'s `dnd` and `mime` crates.

## Current audit status

The latest observed `cargo deny check licenses` failure is caused by
**upstream package metadata deficiencies**, not by an unidentified license
text in the audited dependency sources.

The following are explicitly *not* being treated as license approvals merely
because their SPDX identifiers appear in the global allowlist:

- `cosmic-config`
- `cosmic-config-derive`
- `cosmic-theme`
- `cosmic-settings-daemon`
- `dnd`
- `mime`

Their evidence is recorded above and must be revalidated when dependency
revisions change.

## Re-audit commands

Run:

```fish
cargo deny check licenses
```

For a specific dependency:

```fish
cargo tree -i <crate>@<version>
```

For a Git dependency, identify the exact revision in `Cargo.lock`, then inspect
its Git object and license files.

Examples:

```fish
git -C ~/.cargo/git/db/libcosmic-* show <revision>:LICENSE
```

```fish
git -C ~/.cargo/git/db/dbus-settings-bindings-* show <revision>:LICENSE.md
```

## Recommended upstream fixes

The cleanest permanent solution is for the upstream projects to declare
license metadata for every package.

Examples, subject to maintainer confirmation:

- libcosmic child crates: `license = "MPL-2.0"`
- `cosmic-settings-daemon`: `license = "MPL-2.0"`
- `dnd`: `license = "MIT"`
- `mime`: `license = "MIT"`

These values should only be added upstream after the maintainers confirm that
they accurately describe the intended licensing of those packages.

## Embedded Third-Party Assets

### Weather Icons

- Component: Weather Icons font
- Author: Erik Flowers
- License: SIL Open Font License 1.1
- Usage: Weather condition icons
- Location: resources/fonts/WeatherIcons.ttf
- License text: WEATHER_ICONS_LICENSE.txt

## Scope

This file records the dependency license investigation performed for Toodle
at the revisions documented above. It is not a legal opinion and does not
replace the full license texts distributed by the upstream projects.
