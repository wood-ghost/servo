#!/usr/bin/env bash
set -euo pipefail

# Let bindgen find the GCC 13 C++ headers unless the caller supplies other arguments.
export BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward}"

# Match the project's established Rust/Verus verification workflow.
exec cargo +1.99.1 verus focus \
    -p servo-net-traits \
    -p servo-script \
    --locked --offline -- \
    --compile \
    --no-lifetime \
    --no-trait-conflicts \
    --expand-errors \
    --multiple-errors 20 \
    "$@"
