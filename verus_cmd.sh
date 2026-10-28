# for mime_classifier only
clear && cargo +1.98.1 verus focus \
    -p servo-net-traits -- \
    --no-lifetime \
    --no-trait-conflicts \
    --expand-errors \
    --multiple-errors 20

# for mime_classifier and processingoptions
clear &&
BINDGEN_EXTRA_CLANG_ARGS="-isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward" \
cargo +1.99.1 verus focus \
  -p servo-net-traits \
  -p servo-script \
  --locked -- \
  --compile \
  --no-lifetime \
  --no-trait-conflicts \
  --expand-errors \
  --multiple-errors 20
