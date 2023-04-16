#!/usr/bin/env bash
# This is the second in line to publish the abstract packages. Chceck the base [`Cargo.toml`] and uncomment the version of `abstract-boot`.
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

function print_usage() {
  echo "Usage: $0 [-h|--help]"
  echo "Publishes crates to crates.io."
}

if [ $# = 1 ] && { [ "$1" = "-h" ] || [ "$1" = "--help" ] ; }
then
    print_usage
    exit 1
fi

ALL_PACKAGES="abstract-api abstract-app abstract-ibc-host"

for pack in $ALL_PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing $pack"
    cargo publish
  )
done

echo "Everything is published!"

VERSION=$(grep -A1 "\[workspace.package\]" Cargo.toml | awk -F'"' '/version/ {print $2}');
git tag v"$VERSION"
git push origin v"$VERSION"
