#!/usr/bin/env bash
set -euo pipefail
repo_dir="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_dir"
mode="${1:-dev}"
case "$mode" in dev|build|install|test) ;; *) echo "Usage: $0 [dev|build|install|test]" >&2; exit 2;; esac
for executable in cargo pnpm cmake python3; do
  command -v "$executable" >/dev/null || { echo "Missing $executable. See docs/notepad.md." >&2; exit 1; }
done
if [[ "$(uname -s)" != Darwin ]]; then echo "This launcher targets macOS." >&2; exit 1; fi
if ! /usr/bin/xcodebuild -version >/dev/null 2>&1; then
  export PATH="$repo_dir/scripts/macos-clt:$PATH"
fi
# The pinned llama library's optional HTTPS helper is incompatible with OpenSSL 4.
# Models are downloaded by Meetily's existing Rust downloader, not this helper.
export CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE
export NEXT_PUBLIC_MEETILY_LOCAL_BUILD=1
target_triple="$(rustc -vV | sed -n 's/^host: //p')"
cargo build --release --locked -p llama-helper --features metal
mkdir -p frontend/src-tauri/binaries
cp target/release/llama-helper "frontend/src-tauri/binaries/llama-helper-$target_triple"
config="src-tauri/tauri.notepad.json"
if [[ "$mode" == test ]]; then config="src-tauri/tauri.notepad-test.json"; fi
cd frontend
if [[ "$mode" == dev ]]; then
  if pgrep -f '^/Applications/meetily.app/Contents/MacOS/meetily' >/dev/null; then
    echo "Quit the installed Meetily app before opening this build with the same library." >&2
    exit 1
  fi
  exec pnpm exec tauri dev --config "$config"
fi
pnpm exec tauri build --config "$config" --bundles app
if [[ "$mode" == install ]]; then
  destination="$HOME/Applications/Meetily Notepad.app"
  mkdir -p "$HOME/Applications"
  if [[ -e "$destination" ]]; then
    echo "A local notepad app already exists at $destination; the new build is in target/release/bundle/macos." >&2
    exit 1
  fi
  ditto "$repo_dir/target/release/bundle/macos/Meetily Notepad.app" "$destination"
  echo "Installed: $destination"
fi
