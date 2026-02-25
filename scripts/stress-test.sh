#!/bin/bash

# Tokens
TOKEN_ONE="test1"
TOKEN_TWO="test2"

# Config
URL="http://localhost:8080/user/orders/create"
REQ_COUNT=500
CONCURRENCY=50

# Headers
HEADER_ONE="Authorization: Bearer $TOKEN_ONE"
HEADER_TWO="Authorization: Bearer $TOKEN_TWO"
CONTENT_TYPE="Content-Type: application/json"

# Payloads
cat > payload1.json <<EOF
{
  "market_id": "898a074c-48da-49e7-90f4-417e6e5e5886",
  "price": 0.4,
  "quantity": 12,
  "side": "BUY",
  "outcome_side": "YES"
}
EOF

cat > payload2.json <<EOF
{
  "market_id": "898a074c-48da-49e7-90f4-417e6e5e5886",
  "price": 0.34,
  "quantity": 12,
  "side": "SELL",
  "outcome_side": "YES"
}
EOF

# Run hey for both payloads concurrently
echo "Starting concurrent stress test with 2 payloads..."

hey -n $REQ_COUNT -c $CONCURRENCY -m POST \
  -H "$HEADER_ONE" \
  -H "$CONTENT_TYPE" \
  -d @payload1.json \
  "$URL" > result_buy.txt &

hey -n $REQ_COUNT -c $CONCURRENCY -m POST \
  -H "$HEADER_TWO" \
  -H "$CONTENT_TYPE" \
  -d @payload2.json \
  "$URL" > result_sell.txt &

# Wait for both to finish
wait

echo "Stress test complete. Results saved to result_buy.txt and result_sell.txt"