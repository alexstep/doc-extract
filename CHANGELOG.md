# Changelog

## 0.2.0

- Cloud-agent setup: `AGENTS.md`, `.cursor/environment.json`, format fixtures, and tests for every supported extractor (including empty, truncated, and Unicode files) under Rust, Node, and Bun.
- CI runs `cargo fmt --check`, Clippy, `cargo test`, Node and Bun tests, and the napi prebuild matrix.
- npm publishing on a `v*` tag uses Trusted Publishing (OIDC). The publish job no longer reads `NPM_TOKEN` or `NODE_AUTH_TOKEN`. The tag version must match `package.json`.
