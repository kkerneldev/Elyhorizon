#!/usr/bin/env bash
# Build the fork's complete app bundle on macOS, without upstream signing infrastructure.
# Usage: elyhorizon.sh LAUNCHER CLIENT VERSION OUTDIR arm64|x64
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
launcher=$1
client=$2
version=$3
out=$(mkdir -p "$4" && cd "$4" && pwd)
arch=$5
case "$arch" in
  arm64) cpu=arm64 ;;
  x64) cpu=x86_64 ;;
  *) echo 'Expected arm64 or x64' >&2; exit 1 ;;
esac
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9]+([.-][A-Za-z0-9]+)*)?$ ]] || { echo 'Invalid release version' >&2; exit 1; }
for binary in "$launcher" "$client"; do
  [[ "$(lipo -archs "$binary")" == "$cpu" ]] || { echo "Wrong binary architecture: $binary" >&2; exit 1; }
done
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
app=$stage/Elyhorizon.app
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$launcher" "$app/Contents/MacOS/horizon-launcher"
cp "$client" "$app/Contents/MacOS/horizon-client"
chmod 755 "$app/Contents/MacOS/"*
cp "$here/../icons/horizon.icns" "$app/Contents/Resources/horizon.icns"
cp "$here/../../LICENSE" "$here/../../THIRD_PARTY_NOTICES.md" "$here/../../docs/elyby-account.md" "$app/Contents/Resources/"
python3 - "$here/Info.plist" "$app/Contents/Info.plist" "$version" <<'PY'
import plistlib
import sys
with open(sys.argv[1], 'rb') as source:
    info = plistlib.load(source)
info.update(CFBundleName='Elyhorizon', CFBundleDisplayName='Elyhorizon',
            CFBundleIdentifier='dev.elyhorizon.launcher',
            CFBundleVersion=sys.argv[3].split('-')[0],
            CFBundleShortVersionString=sys.argv[3].split('-')[0],
            ElyhorizonReleaseVersion=sys.argv[3], LSMinimumSystemVersion='12.0')
with open(sys.argv[2], 'wb') as target:
    plistlib.dump(info, target)
PY
printf 'APPL????' > "$app/Contents/PkgInfo"
plutil -lint "$app/Contents/Info.plist"
# Sign nested executables first, then seal the app. No credentials are needed for ad hoc signatures.
codesign --force --sign - "$app/Contents/MacOS/horizon-client"
codesign --force --sign - "$app/Contents/MacOS/horizon-launcher"
codesign --force --sign - "$app"
codesign --verify --deep --strict "$app"
"$app/Contents/MacOS/horizon-launcher" --version
"$app/Contents/MacOS/horizon-client" --help > /dev/null
base=Elyhorizon-$version-macos-$arch
ditto -c -k --norsrc --noextattr --noqtn --keepParent "$app" "$out/$base.zip"
ln -s /Applications "$stage/Applications"
hdiutil create -volname Elyhorizon -srcfolder "$stage" -format UDZO -ov "$out/$base.dmg"
hdiutil verify "$out/$base.dmg"
echo "Packaged $out/$base.zip and $out/$base.dmg (ad hoc signed, not notarized)"
