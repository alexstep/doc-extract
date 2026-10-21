# Changelog

## 0.2.1

- Security: Rust dependencies updated to close RustSec advisories in the parsers (zip 9, quick-xml 0.42, pdf-extract 0.12, calamine 0.37, html2text 0.17). Unused `docx-rs` and `epub` crates removed.
- `maxBytes` now also caps the total uncompressed size of zip-based formats (DOCX, PPTX, EPUB, ODT, XLSX, ODS), so zip bombs are rejected. Archives with more than 4096 entries are rejected.
- File length is compared as a 64-bit value, so files over 4 GiB cannot bypass the limit on 32-bit targets.
- A poisoned internal lock no longer crashes the process; an oversized working set is rejected up front.
- Dev dependencies updated; CI runs `cargo audit`; release builds use `bun install --frozen-lockfile`.

## 0.2.0

- Cloud-agent setup: `AGENTS.md`, `.cursor/environment.json`, format fixtures, and tests for every supported extractor (including empty, truncated, and Unicode files) under Rust, Node, and Bun.
- CI runs `cargo fmt --check`, Clippy, `cargo test`, Node and Bun tests, and the napi prebuild matrix.
- npm publishing on a `v*` tag uses Trusted Publishing (OIDC). The publish job no longer reads `NPM_TOKEN` or `NODE_AUTH_TOKEN`. The tag version must match `package.json`.
