#!/bin/sh
# Installs Radial Convert on macOS: the app, the `radial` command, and (optionally)
# the converters you pick.
#
#   curl -fsSL https://github.com/OWNER/REPO/releases/latest/download/install.sh | sh
#
# Options (after `sh -s --` when piping):
#   --no-app        install only the `radial` command
#   --no-packs      do not offer the converter picker afterwards
#   --packs a,b     install these packs without asking (e.g. audio,images)
#
# Everything downloaded is checked against the release's SHA256SUMS before it is
# used. Environment overrides: RADIAL_BASE_URL, RADIAL_INSTALL_DIR, RADIAL_APP_DIR.

set -eu

# The release these files come from. Set this once the repo exists; the script
# refuses to run while it still contains the placeholder.
BASE_URL="${RADIAL_BASE_URL:-https://github.com/OWNER/REPO/releases/latest/download}"
CLI_DIR="${RADIAL_INSTALL_DIR:-$HOME/.local/bin}"
APP_ASSET="Radial-Convert-macos-universal.zip"
APP_NAME="Radial Convert.app"

install_app=1
offer_packs=1
packs=""

say() { printf '%s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --no-app) install_app=0 ;;
    --no-packs) offer_packs=0 ;;
    --packs)
      [ $# -ge 2 ] || die "--packs needs a value, e.g. --packs audio,images"
      packs="$2"; shift ;;
    -h|--help) sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) die "unknown option: $1" ;;
  esac
  shift
done

[ "$(uname -s)" = "Darwin" ] || die "this script is for macOS. On Windows use install.ps1."
case "$BASE_URL" in
  *OWNER/REPO*) die "this script has not been pointed at a release yet. Set RADIAL_BASE_URL to the release download address." ;;
esac

case "$(uname -m)" in
  arm64) arch="aarch64" ;;
  x86_64) arch="x86_64" ;;
  *) die "unsupported processor: $(uname -m)" ;;
esac

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

fetch() { curl -fsSL --retry 3 -o "$2" "$BASE_URL/$1" || die "could not download $1 from $BASE_URL"; }

# Succeeds only if the file's SHA-256 equals the entry for $2 in SHA256SUMS.
verify() {
  want="$(awk -v n="$2" '$2 == n || $2 == "*" n { print $1 }' "$tmp/SHA256SUMS" | head -n 1)"
  [ -n "$want" ] || die "$2 is not listed in SHA256SUMS"
  got="$(shasum -a 256 "$1" | awk '{ print $1 }')"
  [ "$want" = "$got" ] || die "checksum mismatch for $2 (expected $want, got $got). Nothing was installed."
}

say "Downloading checksums"
fetch SHA256SUMS "$tmp/SHA256SUMS"

cli_asset="radial-macos-$arch"
say "Downloading $cli_asset"
fetch "$cli_asset" "$tmp/$cli_asset"
verify "$tmp/$cli_asset" "$cli_asset"

if [ "$install_app" -eq 1 ]; then
  say "Downloading $APP_ASSET"
  fetch "$APP_ASSET" "$tmp/$APP_ASSET"
  verify "$tmp/$APP_ASSET" "$APP_ASSET"
fi

# Nothing is installed until every download has been verified.
mkdir -p "$CLI_DIR"
cp "$tmp/$cli_asset" "$CLI_DIR/radial"
chmod +x "$CLI_DIR/radial"
say "Installed radial to $CLI_DIR/radial"

if [ "$install_app" -eq 1 ]; then
  app_dir="${RADIAL_APP_DIR:-}"
  if [ -z "$app_dir" ]; then
    if [ -w /Applications ]; then app_dir="/Applications"; else app_dir="$HOME/Applications"; fi
  fi
  mkdir -p "$app_dir"
  ditto -x -k "$tmp/$APP_ASSET" "$tmp/app"
  rm -rf "$app_dir/$APP_NAME"
  ditto "$tmp/app/$APP_NAME" "$app_dir/$APP_NAME"
  say "Installed $APP_NAME to $app_dir"
fi

case ":$PATH:" in
  *":$CLI_DIR:"*) ;;
  *) say ""; say "Add this to your shell profile so \`radial\` is found:"; say "  export PATH=\"$CLI_DIR:\$PATH\"" ;;
esac

if [ -n "$packs" ]; then
  say ""
  "$CLI_DIR/radial" install --packs "$packs" --yes
elif [ "$offer_packs" -eq 1 ]; then
  say ""
  # When piped from curl, stdin is the script itself, so the picker reads the keyboard directly.
  if [ -r /dev/tty ] && : < /dev/tty 2>/dev/null; then
    "$CLI_DIR/radial" install < /dev/tty
  else
    say "Run \`radial install\` to choose which converters to add."
  fi
fi

say ""
say "Done. Open Radial Convert, then hold Shift and drag a file."
