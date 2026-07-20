cargo verus verify \
  -p servo-net-traits \
  --fwd-verus-args-to roots \
  -- \
  --verify-module mime_classifier \
  --expand-errors