#!/usr/bin/env bash
# Idempotent Cloud Agent bootstrap: Rust stable, Node.js LTS (>= 22), Bun, JS deps, native addon.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

ensure_rust() {
  local rustup_bin=""
  if [[ -x "${HOME}/.cargo/bin/rustup" ]]; then
    rustup_bin="${HOME}/.cargo/bin/rustup"
  elif [[ -x /usr/local/cargo/bin/rustup ]]; then
    rustup_bin=/usr/local/cargo/bin/rustup
  else
    curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs \
      | sh -s -- -y --default-toolchain stable --profile minimal --component rustfmt,clippy
    rustup_bin="${HOME}/.cargo/bin/rustup"
  fi

  local cargo_bin
  cargo_bin="$(dirname "$rustup_bin")"
  export PATH="${cargo_bin}:${PATH}"

  "$rustup_bin" toolchain install stable --profile minimal --component rustfmt,clippy
  "$rustup_bin" default stable

  # Login and non-login shells both see these. Wrappers exec the real binaries so
  # rustc still finds its sysroot (a plain symlink would not).
  local tool
  for tool in rustup rustc cargo rustdoc rustfmt cargo-fmt cargo-clippy clippy-driver; do
    if [[ -x "${cargo_bin}/${tool}" ]]; then
      sudo tee "/usr/local/bin/${tool}" >/dev/null <<EOF
#!/bin/sh
export PATH="${cargo_bin}:\$PATH"
exec "${cargo_bin}/${tool}" "\$@"
EOF
      sudo chmod 755 "/usr/local/bin/${tool}"
    fi
  done
}

ensure_node() {
  if command -v node >/dev/null 2>&1; then
    local major
    major="$(node -p 'Number(process.versions.node.split(".")[0])')"
    if [[ "$major" -ge 22 ]]; then
      return
    fi
  fi

  local version tmp
  version="$(
    curl -fsSL https://nodejs.org/dist/index.json \
      | python3 -c 'import json,sys; rels=json.load(sys.stdin); print(next(r["version"] for r in rels if r.get("lts")))'
  )"
  tmp="$(mktemp -d)"
  curl -fsSL "https://nodejs.org/dist/${version}/node-${version}-linux-x64.tar.xz" \
    | tar -xJ -C "$tmp"
  sudo cp -a "${tmp}/node-${version}-linux-x64/." /usr/local/
  rm -rf "$tmp"
}

ensure_bun() {
  if [[ ! -x "${HOME}/.bun/bin/bun" ]]; then
    curl -fsSL https://bun.sh/install | bash
  fi
  sudo ln -sf "${HOME}/.bun/bin/bun" /usr/local/bin/bun
}

ensure_rust
ensure_node
ensure_bun

hash -r || true
bun install --frozen-lockfile
bun run build
