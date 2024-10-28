#!/bin/bash

# These variables needs to be configured
NODE_ADDRESS="${NODE_ADDRESS:-http://3.14.48.188:7777}"
ACCOUNT_HASH="${ACCOUNT_HASH:-account-hash-80ac9361496a02b1408cc019a7876d052ed8d00cc34f4fa8728e9e2a781425a6}"
CHAIN_NAME="${CHAIN_NAME:-dev-net}"
SECRET_KEY_PATH="${SECRET_KEY_PATH:-/Users/raveena/work/ed25519_keys/secret_key.pem}"

get_state_root_hash() {
  OUTPUT=$(cargo run --release -- get-state-root-hash --node-address "$NODE_ADDRESS")
  STATE_ROOT_HASH=$(echo "$OUTPUT" | jq -r '.result.state_root_hash')

  if [ -z "$STATE_ROOT_HASH" ]; then
    echo "Failed to retrieve state_root_hash. Please check the cargo command and the node address."
    exit 1
  fi
  echo "Retrieved state_root_hash: $STATE_ROOT_HASH"
}

get_package_hash() {
  QUERY_OUTPUT=$(cargo run --release -- query-state \
    --node-address "$NODE_ADDRESS" \
    --state-root-hash "$STATE_ROOT_HASH" \
    --key "$ACCOUNT_HASH" \
    -q "contract_hash_asset_bridge")

  FULL_PACKAGE_HASH=$(echo "$QUERY_OUTPUT" | jq -r '.result.stored_value.AddressableEntity.package_hash')
  PACKAGE_HASH="${FULL_PACKAGE_HASH#package-}"
  echo "$PACKAGE_HASH"

  if [ -z "$PACKAGE_HASH" ]; then
    echo "Failed to retrieve package_hash"
    exit 1
  fi
  echo "Retrieved package_hash: $PACKAGE_HASH"
}

deploy() {
  ENTRY_POINT=$1

  # Set session arguments based on the entry point
  case $ENTRY_POINT in
  "init") SESSION_ARGS='[]' ;;
  "update_config")
    SESSION_ARGS='[{"name":"gateway_address","type":"String","value":"casper"}]'
    ;;
  "set_dapp_metadata")
    SESSION_ARGS='[
    {"name":"fee_payer_address","type":"String","value":"account-hash-0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"}
    ]'
    ;;
  "set_liquidity_pool_multi")
    SESSION_ARGS='[
    {"name":"tokens",
      "type":{"List":"Key"},
      "value":["hash-0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20","hash-2122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f40"]
    },
    {"name":"lp_tokens",
      "type":{"List":"Key"},
      "value":[
      "hash-4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60",
      "hash-6162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f80"]
    }
    ]'
    ;;
  "set_whitelist_token_multi")
    SESSION_ARGS='[
    { 
      "name":"tokens",
      "type":{"List":"Key"},
      "value":["hash-0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20","hash-2122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f40"]
    },
    {
      "name":"types",
      "type":{"List":"U64"},
      "value":[1,2]
    }
    ]'
    ;;
  "transfer_token")
    SESSION_ARGS='[
    {"name":"dest_chain_id","type":"String","value":"ethereum"},
    {"name":"src_token","type":"Key","value":"hash-0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"},
    {"name":"recipient","type":"Key","value":"account-hash-0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"},
    {"name":"src_token_amount","type":"U256","value":"1000"},
    {"name":"partner_id","type":"U256","value":"1"},
    {"name":"deposit_id","type":"U256","value":"1"}
    ]'
    ;;
  "handle_message") SESSION_ARGS='[]' ;;
  *)
    echo "Invalid entry point"
    exit 1
    ;;
  esac

  cargo run --release -- put-deploy \
    --node-address "$NODE_ADDRESS" \
    --chain-name "$CHAIN_NAME" \
    --secret-key "$SECRET_KEY_PATH" \
    --payment-amount 1000000 \
    --session-package-hash "$PACKAGE_HASH" \
    --session-entry-point "$ENTRY_POINT" \
    --session-args-json "$SESSION_ARGS"
}

# Main script execution
if [ $# -eq 0 ]; then
  echo "No entry point provided. Usage: $0 <entry_point>"
  exit 1
fi

get_state_root_hash
sleep 5
get_package_hash
deploy "$1"
