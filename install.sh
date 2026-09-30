#!/usr/bin/env bash
# Install wisp: build from a clone (or download the latest release), add a launcher entry.
#
#   curl -fsSL https://raw.githubusercontent.com/Comninos/wisp/master/install.sh | bash
#   curl -fsSL …/install.sh | sudo bash -s -- --system
#   ./install.sh

set -euo pipefail

RELEASE_URL="${WISP_RELEASE_URL:-https://github.com/Comninos/wisp/releases/latest/download/wisp-linux-x86_64}"
SYSTEM="${WISP_SYSTEM:-0}"

say() { printf '%s\n' "$*"; }
warn() { printf 'wisp install: %s\n' "$*" >&2; }
die() { printf 'wisp install: %s\n' "$*" >&2; exit 1; }

usage() {
    cat <<'EOF'
Usage: install.sh [options]

  --system     Install to /usr/local (requires root)
  -h, --help   Show this help

Env: WISP_SYSTEM=1  WISP_RELEASE_URL=<url>
EOF
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "need '$1'"
}

case "$(uname -s 2>/dev/null || true)" in
    Linux) ;;
    *) die "this installer is for Linux; on Windows use install.ps1" ;;
esac

while [[ $# -gt 0 ]]; do
    case "$1" in
        --system) SYSTEM=1 ;;
        -h|--help) usage; exit 0 ;;
        *) die "unknown option: $1 (try --help)" ;;
    esac
    shift
done

[[ "${XDG_SESSION_TYPE:-wayland}" == "wayland" ]] || warn "wisp is built for Wayland; see examples.md for X11"

here=""
if [[ -n "${BASH_SOURCE[0]:-}" ]]; then
    here="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || true)"
fi

fetch_binary() {
    local dest="$1"
    if [[ -n "$here" && -f "${here}/Cargo.toml" ]]; then
        need_cmd cargo
        say "building from ${here}"
        cargo build --release --manifest-path "${here}/Cargo.toml"
        cp "${here}/target/release/wisp" "$dest"
        return
    fi
    need_cmd curl
    curl -fsSL "$RELEASE_URL" -o "$dest"
}

if [[ "$SYSTEM" == "1" ]]; then
    [[ "$(id -u)" -eq 0 ]] || die "--system / WISP_SYSTEM=1 requires root (try sudo)"
    prefix="/usr/local"
else
    prefix="${HOME}/.local"
fi
bin_path="${prefix}/bin/wisp"
desktop_path="${prefix}/share/applications/wisp.desktop"

say ""
say "Install wisp to ${bin_path}"
say ""

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
fetch_binary "$tmp"
mkdir -p "${prefix}/bin" "${prefix}/share/applications"
install -m 0755 "$tmp" "$bin_path"
say "installed ${bin_path}"

cat >"$desktop_path" <<EOF
[Desktop Entry]
Type=Application
Name=wisp
Comment=Scratch pad that never saves
Exec=${bin_path}
Terminal=false
Categories=Utility;TextEditor;
StartupWMClass=wisp
EOF
say "added launcher ${desktop_path}"

say ""
if [[ ":${PATH}:" != *":${prefix}/bin:"* ]]; then
    warn "${prefix}/bin is not on PATH; launch wisp from your app menu or add it to PATH"
fi
say "run:  wisp"
if [[ -n "$here" && -f "${here}/Cargo.toml" ]]; then
    say "edit ${here}/src/main.rs, then rerun ./install.sh"
fi
