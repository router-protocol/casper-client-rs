#!/bin/bash

ACCOUNT_HASH="${ACCOUNT_HASH:-account-hash-80ac9361496a02b1408cc019a7876d052ed8d00cc34f4fa8728e9e2a781425a6}"

# Function to retrieve state_root_hash
get_state_root_hash() {
  OUTPUT=$(cargo run --release -- get-state-root-hash --node-address http://3.14.48.188:7777)
  STATE_ROOT_HASH=$(echo "$OUTPUT" | jq -r '.result.state_root_hash')

  if [ -z "$STATE_ROOT_HASH" ]; then
    echo "Failed to retrieve state_root_hash. Please check the cargo command and the node address."
    exit 1
  fi
  echo "Retrieved state_root_hash: $STATE_ROOT_HASH"
}

# Main script execution
if [ $# -eq 0 ]; then
  echo "No input provided. Usage: $0 <input_to_append_after_slash>"
  exit 1
fi

USER_INPUT=$1

get_state_root_hash

cargo run --release -- query-state \
  --node-address http://3.14.48.188:7777 \
  --state-root-hash "$STATE_ROOT_HASH" \
  --key "$ACCOUNT_HASH" \
  -q "asset_forwarder_contract_hash/$USER_INPUT"
