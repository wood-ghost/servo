#!/usr/bin/env bash
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -euo pipefail

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
revision=dea85441338f13fbab017b7c0ebafc9ad11464ce
# Dependency sources survive cargo/mach clean; build artifacts remain in target.
source_dir="$root/.verus-source"
mode=${1:-verify}

case "$mode" in
    prepare)
        if [[ $# != 2 ]]; then
            printf 'Usage: bash verus_cmd.sh prepare /path/to/verus-git-clone\n' >&2
            exit 2
        fi
        # Archive the pinned object, independent of the clone's checked-out branch.
        git -C "$2" cat-file -e "$revision^{commit}"
        if [[ -e "$source_dir" ]]; then
            printf 'Snapshot already exists: %s\n' "$source_dir" >&2
            exit 1
        fi
        temporary=$(mktemp -d "$root/.verus-source.XXXXXX")
        trap 'rm -rf -- "$temporary"' EXIT
        git -C "$2" archive "$revision" | tar -x -C "$temporary"
        # Portable atomic rename: unlike GNU mv -T, this also works on macOS.
        python3 -c 'import os, sys; os.rename(sys.argv[1], sys.argv[2])' "$temporary" "$source_dir"
        trap - EXIT
        printf 'Prepared Verus dependency source %s in %s\n' "$revision" "$source_dir"
        exit 0
        ;;
    verify|native)
        if [[ $# -gt 1 ]]; then
            printf 'Usage: bash verus_cmd.sh [verify|native]\n' >&2
            exit 2
        fi
        ;;
    *)
        printf 'Usage: bash verus_cmd.sh {prepare /path/to/verus-git-clone|verify|native}\n' >&2
        exit 2
        ;;
esac

if [[ ! -f "$source_dir/source/vstd/Cargo.toml" ]]; then
    printf 'First run: bash verus_cmd.sh prepare /path/to/verus-git-clone\n' >&2
    exit 1
fi

# Linux/GCC 13 default used for the recorded checks; other installations can set
# BINDGEN_EXTRA_CLANG_ARGS explicitly, including to the empty string.
export BINDGEN_EXTRA_CLANG_ARGS=${BINDGEN_EXTRA_CLANG_ARGS-"-isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward"}
cd -- "$root"

if [[ "$mode" == native ]]; then
    # Exercise normal dependency resolution, without a command-line Cargo config.
    exec cargo +1.97.1 check -p servo-script --lib --locked --offline
fi

# Requires the recorded installed Verus/cargo-verus on PATH. Preparing the source
# snapshot does not build that verifier or establish its binary provenance.
exec cargo +1.99.1 verus focus -p servo-net-traits -p servo-script --locked -- \
    --compile --no-lifetime --no-trait-conflicts --expand-errors --multiple-errors 20
