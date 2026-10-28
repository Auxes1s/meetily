#!/usr/bin/env bash
# Run in a Linux GitHub Codespace. Produces an unsigned, CPU-only Windows installer.
set -Eeuo pipefail
trap 'printf "\nBuild failed at line %s. See %s/build.log\n" "$LINENO" "$OUT" >&2' ERR

SOURCE_REF="${SOURCE_REF:-a02f6284b8ae12099a44f961b05a6a9bd72e75a1}"
REPO_URL="${REPO_URL:-https://github.com/Auxes1s/meetily.git}"
OUT="${OUT:-$PWD/meetily-windows-output}"
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"
exec > >(tee -a "$OUT/build.log") 2>&1
[[ "$(uname -s)" == Linux && "$(uname -m)" == x86_64 ]] || { echo 'Use an x64 Linux Codespace.'; exit 1; }
command -v sudo >/dev/null
printf 'Building source %s; output: %s\n' "$SOURCE_REF" "$OUT"
echo 'This builds CPU-only transcription. No Vulkan/CUDA acceleration is included.'

sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  build-essential clang llvm lld libclang-dev cmake ninja-build nsis \
  pkg-config libssl-dev curl ca-certificates git unzip python3 python3-venv \
  cabextract wine64
export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v rustup >/dev/null; then
  curl --proto '=https' --tlsv1.2 -fsS https://sh.rustup.rs -o "$OUT/rustup-init.sh"
  sh "$OUT/rustup-init.sh" -y --profile minimal --no-modify-path
fi
rustup toolchain install stable --profile minimal
export RUSTUP_TOOLCHAIN=stable
rustup target add x86_64-pc-windows-msvc

python3 -m venv "$OUT/tools"
"$OUT/tools/bin/pip" install 'cargo-xwin==0.23.1' pefile
export PATH="$OUT/tools/bin:$PATH"
if command -v corepack >/dev/null; then
  PNPM=(corepack pnpm@9.15.9)
else
  command -v npm >/dev/null || { echo 'Select a Codespace image with Node.js installed.'; exit 1; }
  PNPM=(npx --yes pnpm@9.15.9)
fi

WORK="$(mktemp -d "${TMPDIR:-/tmp}/meetily-windows.XXXXXXXX")"
echo "Disposable source checkout: $WORK"
git -C "$WORK" init -q
git -C "$WORK" remote add origin "$REPO_URL"
git -C "$WORK" fetch --depth 1 origin "$SOURCE_REF"
git -C "$WORK" checkout --detach FETCH_HEAD
cd "$WORK"
git rev-parse HEAD > "$OUT/source-commit.txt"

# The upstream build executes the bundled FFmpeg to validate it. Use Wine when
# building a Windows executable on Linux, retaining the actual version check.
python3 - <<'PY'
from pathlib import Path
p = Path('frontend/src-tauri/build/ffmpeg.rs')
s = p.read_text()
old = 'match std::process::Command::new(path)\n        .arg("-version")\n        .output()'
new = '''let cross_windows = std::env::var("TARGET").unwrap_or_default().contains("windows")
        && !std::env::var("HOST").unwrap_or_default().contains("windows");
    let mut command = if cross_windows {
        let mut c = std::process::Command::new("wine");
        c.arg(path);
        c
    } else {
        std::process::Command::new(path)
    };
    match command.arg("-version").output()'''
assert s.count(old) == 1, 'FFmpeg verifier changed; review the cross-build adaptation.'
p.write_text(s.replace(old, new))
PY
git diff > "$OUT/cross-build.patch"

export WINEDEBUG=-all
export WINEPREFIX="$OUT/wine"
if ! command -v wine >/dev/null; then
  mkdir -p "$OUT/bin"
  printf '#!/bin/sh\nexec /usr/lib/wine/wine64 "$@"\n' > "$OUT/bin/wine"
  chmod +x "$OUT/bin/wine"
  export PATH="$OUT/bin:$PATH"
fi
wine --version
export XWIN_CACHE_DIR="$OUT/xwin-cache"
export XWIN_ARCH=x86_64
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export CMAKE_BUILD_PARALLEL_LEVEL="$CARGO_BUILD_JOBS"
export CMAKE_GENERATOR=Ninja
export CMAKE_PROJECT_INCLUDE="$WORK/.github/force-portable-ggml.cmake"
export CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE
export RUSTFLAGS='-C target-cpu=x86-64-v2'
export NEXT_PUBLIC_MEETILY_LOCAL_BUILD=1
export LIBCLANG_PATH="$(llvm-config --libdir)"

# Bundle Microsoft's runtime DLLs next to the app and ONNX Runtime. Extract the
# official redistributable without executing its Windows installer.
mkdir -p frontend/src-tauri/binaries/msvc-runtime "$OUT/redist"
curl -fL --retry 3 https://aka.ms/vs/17/release/vc_redist.x64.exe -o "$OUT/redist/vc_redist.x64.exe"
cabextract -q -d "$OUT/redist/outer" "$OUT/redist/vc_redist.x64.exe"
for cab in "$OUT"/redist/outer/*; do
  if [[ -f "$cab" && "$(head -c 4 "$cab")" == MSCF ]]; then
    mkdir -p "$cab.contents"
    cabextract -q -d "$cab.contents" "$cab"
  fi
done
"$OUT/tools/bin/python" - "$OUT/redist" <<'PY'
import hashlib, json, shutil, sys
from pathlib import Path
import pefile
dest = Path('frontend/src-tauri/binaries/msvc-runtime')
manifest = []
for p in Path(sys.argv[1]).rglob('*'):
    if not p.is_file() or p.stat().st_size < 64:
        continue
    with p.open('rb') as f:
        if f.read(2) != b'MZ':
            continue
    try:
        pe = pefile.PE(str(p), fast_load=False)
        if pe.FILE_HEADER.Machine != 0x8664:
            continue
        for group in getattr(pe, 'FileInfo', []):
            for item in group:
                for table in getattr(item, 'StringTable', []):
                    name = table.entries.get(b'OriginalFilename', b'').decode(errors='replace').strip('\0')
                    if name.lower().endswith('.dll') and Path(name).name == name:
                        target = dest / name.lower()
                        shutil.copyfile(p, target)
                        manifest.append({'file': target.name, 'sha256': hashlib.sha256(target.read_bytes()).hexdigest()})
    except pefile.PEFormatError:
        pass
for name in ('vcruntime140.dll', 'vcruntime140_1.dll', 'msvcp140.dll'):
    assert (dest / name).is_file(), f'Missing Microsoft runtime: {name}'
(dest / 'runtime-sha256.json').write_text(json.dumps(manifest, indent=2))
PY

"${PNPM[@]}" --dir frontend install --frozen-lockfile
cargo xwin build --release --locked --target x86_64-pc-windows-msvc -p llama-helper
mkdir -p frontend/src-tauri/binaries
cp target/x86_64-pc-windows-msvc/release/llama-helper.exe \
  frontend/src-tauri/binaries/llama-helper-x86_64-pc-windows-msvc.exe

cat > frontend/src-tauri/tauri.codespaces.json <<'JSON'
{
  "bundle": {
    "targets": ["nsis"],
    "createUpdaterArtifacts": false,
    "resources": {
      "templates": "templates",
      "binaries/onnxruntime": ".",
      "binaries/msvc-runtime": "."
    },
    "windows": {"signCommand": null, "nsis": {"installMode": "currentUser"}}
  }
}
JSON
(
  cd frontend
  "${PNPM[@]}" exec tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc \
    --config src-tauri/tauri.v0-candidate.json --config src-tauri/tauri.codespaces.json \
    --bundles nsis -- --locked
)
node .github/verify-portable-ggml.cjs target/x86_64-pc-windows-msvc/release
shopt -s nullglob
installers=(target/x86_64-pc-windows-msvc/release/bundle/nsis/*-setup.exe)
[[ ${#installers[@]} == 1 ]] || { echo 'Expected exactly one NSIS installer.'; exit 1; }
cp "${installers[0]}" "$OUT/"
(
  cd "$OUT"
  sha256sum ./*-setup.exe > SHA256SUMS
)
printf '\nInstaller ready in %s\n' "$OUT"
echo 'Download the *-setup.exe and SHA256SUMS using the Codespaces file explorer.'
echo 'This is an unsigned CPU-only cross-build. Windows launch and upgrade verification are still required.'
