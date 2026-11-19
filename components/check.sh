#!/usr/bin/env bash
set -euo pipefail

# Match the project's established Rust/Verus verification workflow.
exec cargo +1.99.1 verus focus -p servo-net-traits --locked --offline -- \
    --compile \
    --no-lifetime \
    --no-trait-conflicts \
    --expand-errors \
    --multiple-errors 20 \
    "$@"
