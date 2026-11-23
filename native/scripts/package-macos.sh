#!/bin/bash
set -euo pipefail
native_dir="$(cd "$(dirname "$0")/.." && pwd)"
cd "$native_dir"
cargo build --locked --release --target aarch64-apple-darwin
artifact_dir="$native_dir/artifacts/AnatomyNative.xcframework"
package_dir="$native_dir/artifacts/macos-arm64"
mkdir -p "$package_dir"
# Rust staticlib bundles std with globally named runtime symbols. Merge first
# so references are resolved inside one object, then expose only our C ABI.
# Apple's nmedit manual documents this ld -r + nmedit interface pattern.
cat > "$package_dir/exports.txt" <<'SYMBOLS'
_anatomy_renderer_create
_anatomy_renderer_load_scene
_anatomy_renderer_set_control
_anatomy_renderer_encode
_anatomy_renderer_state_json
_anatomy_renderer_last_error
_anatomy_string_free
_anatomy_renderer_destroy
SYMBOLS
sdk_version="$(xcrun --sdk macosx --show-sdk-version)"
# The current Mac client and installed Rust std target macOS 27.
xcrun ld -r -arch arm64 -platform_version macos 27.0 "$sdk_version" \
  -all_load "$native_dir/target/aarch64-apple-darwin/release/libanatomy_native.a" \
  -o "$package_dir/anatomy.o"
xcrun nmedit -s "$package_dir/exports.txt" "$package_dir/anatomy.o"
xcrun libtool -static -o "$package_dir/libanatomy_native.a" "$package_dir/anatomy.o"
xcrun nm -gU "$package_dir/libanatomy_native.a" > "$package_dir/defined-symbols.txt"
python3 - "$package_dir" <<'PY_SYMBOLS'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
expected = set((p / 'exports.txt').read_text().splitlines())
actual = {line.split()[-1] for line in (p / 'defined-symbols.txt').read_text().splitlines() if len(line.split()) == 3}
if actual != expected:
    raise SystemExit(f'Native ABI export mismatch: {actual ^ expected}')
PY_SYMBOLS
# Remove only this script's prior generated XCFramework, never source or user data.
if [ -d "$artifact_dir" ]; then
  python3 - "$artifact_dir" <<'PY'
import pathlib, shutil, sys
path = pathlib.Path(sys.argv[1])
assert path.name == 'AnatomyNative.xcframework' and path.parent.name == 'artifacts'
shutil.rmtree(path)
PY
fi
mkdir -p "$native_dir/artifacts"
xcodebuild -create-xcframework \
  -library "$package_dir/libanatomy_native.a" \
  -headers "$native_dir/include" \
  -output "$artifact_dir"
printf '%s\n' "PASS AnatomyNative macOS arm64 XCFramework: $artifact_dir"
