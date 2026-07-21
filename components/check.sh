cargo +1.96.0 verus focus \
  -p servo-net-traits \
  -- \
  --no-lifetime \
  --verify-module mime_classifier \
  --expand-errors