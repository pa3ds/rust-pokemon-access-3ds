#!/bin/sh
# Build pa3ds for release on macOS or Linux and pack it into one archive a player unpacks and
# runs: the same contents as tools/package.ps1's Windows zip, for this system.
#
#   macOS (arm64):   target/package/pa3ds-<version>-macos.zip
#   Linux (x86_64):  target/package/pa3ds-<version>-linux.tar.gz
#
# holding pa3ds (the program, with Prism built in), data/ (the .ftl files alone), README.md,
# manual/, CHANGELOG.md and licenses/, each as package.ps1 describes them. On macOS the binary is
# signed ad hoc, which Apple silicon needs to run it at all; it is not notarised.
#
# Usage: sh tools/package.sh [--skip-tests]
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

case "$(uname -s)" in
    Darwin) platform=macos ;;
    Linux) platform=linux ;;
    *) echo "package.sh packs for macOS or Linux; Windows is package.ps1" >&2; exit 1 ;;
esac

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
name="pa3ds-$version-$platform"

if [ "${1:-}" != "--skip-tests" ]; then
    echo "== tests"
    cargo test --lib --locked
fi
# Only a packaged pa3ds checks GitHub for a newer release when it starts (src/update.rs).
echo "== release build"
PA3DS_RELEASE=1 cargo build --release --locked --bin pa3ds

release="$root/target/release"
[ -f "$release/pa3ds" ] || { echo "the build left no pa3ds in $release" >&2; exit 1; }
# A start that prints its help reads nothing and says nothing, and proves the executable runs.
echo "== the executable starts"
"$release/pa3ds" --help > /dev/null

package="$root/target/package"
stage="$package/$name"
rm -rf "$stage" "$package/$name.zip" "$package/$name.tar.gz"
mkdir -p "$stage/licenses"

cp "$release/pa3ds" "$stage/pa3ds"
chmod 755 "$stage/pa3ds"
if [ "$platform" = macos ]; then
    codesign -s - --force "$stage/pa3ds"
fi
# Every .ftl, each at its path under data/; the rest of data/ is built into pa3ds (#16).
(cd data && find . -type f -name '*.ftl') | while IFS= read -r file; do
    mkdir -p "$stage/data/$(dirname "$file")"
    cp "data/$file" "$stage/data/$file"
done
cp -R manual "$stage/manual"
cp README.md CHANGELOG.md "$stage/"
cp LICENSE "$stage/licenses/pa3ds.txt"
# Prism's LICENSES/<library>/<file> becomes licenses/<library>.<ext>, as package.ps1 does.
# vendor/prism/LICENSE is left out: it is LICENSES/prism's MPL text again.
prism="$root/vendor/prism"
cp "$prism/NOTICE" "$stage/licenses/prism-notice.txt"
for library in "$prism"/LICENSES/*/; do
    library=${library%/}
    for file in "$library"/*; do
        [ -f "$file" ] || continue
        base=${file##*/}
        case "$base" in
            *.*) stem=${base%.*}; extension=".${base##*.}" ;;
            *) stem=$base; extension=.txt ;;
        esac
        suffix=
        [ "$stem" = AUTHORS ] && suffix=-authors
        target="$stage/licenses/${library##*/}$suffix$extension"
        [ -e "$target" ] && { echo "two of Prism's licence files would both be $target" >&2; exit 1; }
        cp "$file" "$target"
    done
done

if [ "$platform" = macos ]; then
    echo "== zip"
    archive="$package/$name.zip"
    (cd "$package" && zip -qr "$name.zip" "$name")
else
    echo "== tar"
    archive="$package/$name.tar.gz"
    tar -czf "$archive" -C "$package" "$name"
fi
rm -rf "$stage"

echo "wrote $archive ($(du -h "$archive" | cut -f1))"
