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
    -q "contract_hash_asset_forwarder")

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

  case $ENTRY_POINT in
  "init") ;;
  "set_dest_details_test")
    SESSION_ARGS='[
    {"name":"dest_chain_id","type":"String","value":"polygon"},
    {"name":"domain_id","type":"U128","value":"1"},
    {"name":"fee","type":"U256","value":"1"},
    {"name":"is_set","type":"Bool","value":false}
    ]'
    ;;
  "set_dest_details")
    SESSION_ARGS='[
    {"name":"dest_chain_id","type":{"List":"String"},"value":["polygon"]},
    {"name":"domain_ids","type":{"List":"U128"},"value":["1"]},
    {"name":"fees","type":{"List":"U256"},"value":["2"]},
    {"name":"is_set","type":{"List":"Bool"},"value":[false]}
    ]'
    ;;
  "i_deposit")
    SESSION_ARGS='[
        {"name":"partner_id","type":"U128","value":"1"},
        {"name":"src_token","type":"Key","value":"entity-contract-ec301e17c49ee4d18fc2d3f3766fce82389edac756b2f85aef31a8422414289a"},
        {"name":"refund_recipient","type":"Key","value":"account-hash-0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"},
        {"name":"address","type":"Key","value":"account-hash-a4628515772103ba1174bcf28cc99a8785d291c5192e142f53bd23ecfa55556a"},
        {"name":"amount","type":"U512","value":"1"},
        {"name":"dest_chain_id","type":"String","value":"ethereum"},
        {"name":"dest_token","type":"Key","value":"entity-contract-a26eba1eed80d8248d15a619f2395b7b75e79baff4873a0486052dec1fa1b4c1"},
        {"name":"dest_amount","type":"U128","value":"90000"},
        {"name":"recipient","type":"Key","value":"account-hash-a4628515772103ba1174bcf28cc99a8785d291c5192e142f53bd23ecfa55556a"}
    ]'
    ;;
  "i_deposit_with_message")
    SESSION_ARGS='[
        {"name":"partner_id","type":"U128","value":"1"},
        {"name":"src_token","type":"Key","value":"entity-contract-ec301e17c49ee4d18fc2d3f3766fce82389edac756b2f85aef31a8422414289a"},
        {"name":"refund_recipient","type":"Key","value":"account-hash-0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"},
        {"name":"amount","type":"U512","value":"1"},
        {"name":"dest_chain_id","type":"String","value":"ethereum"},
        {"name":"dest_token","type":"Key","value":"entity-contract-a26eba1eed80d8248d15a619f2395b7b75e79baff4873a0486052dec1fa1b4c1"},
        {"name":"dest_amount","type":"U128","value":"90000"},
        {"name":"recipient","type":"Key","value":"account-hash-a4628515772103ba1174bcf28cc99a8785d291c5192e142f53bd23ecfa55556a"},
        {"name":"message","type":{"List":"U8"},"value":[72,101,108,108,111]}
    ]'
    ;;
  "i_relay")
    SESSION_ARGS='[
      {"name":"amount","type":"U128","value":"1"},
      {"name":"src_chain_id","type":{"ByteArray":32},"value":"0000000000000000000000000000000000000000000000000000000000000002"},
      {"name":"deposit_id","type":"U128","value":"1"},
      {"name":"dest_token","type":"Key","value":"entity-contract-ec301e17c49ee4d18fc2d3f3766fce82389edac756b2f85aef31a8422414289a"},
      {"name":"recipient","type":"Key","value":"account-hash-a4628515772103ba1174bcf28cc99a8785d291c5192e142f53bd23ecfa55556a"},
      {"name":"forwarder_router_address","type":"String","value":"router123"}
    ]'
    ;;
  "i_relay_with_message")
    SESSION_ARGS='[
      {"name":"amount","type":"U128","value":"1"},
      {"name":"src_chain_id","type":{"ByteArray":32},"value":"0000000000000000000000000000000000000000000000000000000000000002"},
      {"name":"deposit_id","type":"U128","value":"1"},
      {"name":"dest_token","type":"Key","value":"entity-contract-ec301e17c49ee4d18fc2d3f3766fce82389edac756b2f85aef31a8422414289a"},
      {"name":"recipient","type":"Key","value":"account-hash-a4628515772103ba1174bcf28cc99a8785d291c5192e142f53bd23ecfa55556a"},
      {"name":"message","type":{"List":"U8"},"value":[72,101,108,108,111]},
      {"name":"forwarder_router_address","type":"String","value":"router123"}
    ]'
    ;;
  *)
    echo "Invalid entry point"
    exit 1
    ;;

  esac

  cargo run --release -- put-deploy \
    --node-address http://3.14.48.188:7777 \
    --chain-name dev-net \
    --secret-key "$SECRET_KEY_PATH" \
    --payment-amount 1000 \
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
