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
    -q "contract_hash_gateway")

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
  "init")
    SESSION_ARGS='[]'
    ;;
  "set_bridge_fees")
    SESSION_ARGS='[{"name":"new_fees","type":"U128","value":"11"} ]'
    ;;
  "set_current_version")
    SESSION_ARGS='[{"name":"new_version","type":"U256","value":"3"} ]'
    ;;
  "set_dapp_metadata")
    SESSION_ARGS='[
      {"name":"fee_payer_address","type":"String","value":"account-hash-80ac9361496a02b1408cc019a7876d052ed8d00cc34f4fa8728e9e2a781425a6"}
      ]'
    ;;
  "get_deposit_purse")
    SESSION_ARGS='[
    {"name":"recipient","type":"Key","value":"entity-contract-94802906bcefbcb83635684b96031b64aeff7e8608cc4c3ed6d59089788b5f48"}
    ]'
    ;;
  "deposit")
    SESSION_ARGS='[
    {"name":"purse","type":"URef","value":"uref-6aa3bd7b3c7402b260ac69af4c13b2cc0235c09a688fcccf63443a1fa53a66ee-007"},
    {"name":"recipient","type":"Key","value":"entity-contract-fa8ccd522d3fa2601e0fc6f225cd263a4693b7dd93df14842f497b241aa182de"},
    {"name":"amount","type":"U512","value":"10000"}
    ]'
    ;;
  "deposit_into_session")
    SESSION_ARGS='[
    {"name":"deposit_contract_hash","type":"Key","value":"entity-contract-f2cc23d49bf519ef51c86a00186cdd1d4465080728122cf69ea9cdba81a57dab"},
    {"name":"recipient","type":"Key","value":"entity-contract-f2cc23d49bf519ef51c86a00186cdd1d4465080728122cf69ea9cdba81a57dab"},
    {"name":"amount","type":"U512","value":"100"}
    ]'
    ;;
  "i_send")
    SESSION_ARGS='[
	  {"name":"version", "type": "U128", "value": "1"},
	  {"name": "dest_chain_id", "type": "String", "value": "router"},
	  {"name": "request_metadata","type":{"List":"U8"},"value":[0]},
	  {"name":"request_packet","type":{"List":"U8"},"value":[0]},
	  {"name":"route_amount","type":"U512","value":"1"},
    {"name":"route_token","type":"Key","value":"entity-contract-46ab04c6da07e46e3fb6ee71a6b1cdb0c056ef1cbf2ec28e193cfecc6c978fd4"},
    {"name":"recipient","type":"Key","value":"entity-contract-0b21e56b65dfaaf697bed434492d9f88c6052237ad8e97d6d232546b245ecaaa"}
	  ]'
    ;;
  "i_receive")
    SESSION_ARGS='[
	        {"name":"validators","type":{"List":"String"},"value":["020270fecd1f7ae5c1cd53a52c4ca88cd5b76c2926d7e1d831addaa2a64bea9cc3ed"]}, 
	        {"name":"powers","type":{"List":"U64"},"value":[4294967295]},
	        {"name":"valset_nonce","type":"U128","value":"1"},
          {"name":"route_amount","type":"U128","value":"1"},
          {"name": "request_identifier","type":"U128","value":"1"},
	        {"name":"request_timestamp","type":"U128","value":"1752503506"},
	        {"name":"src_chain_id","type":"String","value":"devnet"},
	        {"name":"route_recipient","type":"String","value":"0202fd5f4dd500c5431d5bd173dd228de288b21e83f50abca01b93ee8315f369a264"},
	        {"name":"dest_chain_id","type":"String","value":"router"},
	        {"name":"asm_address","type":"String","value":"0x"},
	        {"name": "request_sender","type":"String","value":"0203246824f3ab4df3eb425f647846f79800f99a793133736c441ad323ab0807286f"},
	        {"name":"handler_address","type":"String","value":"0203246824f3ab4df3eb425f647846f79800f99a793133736c441ad323ab0807286f"},
	        {"name":"packet","type":{"List":"U8"},"value":[22]},
	        {"name":"is_read_call", "type":"Bool","value":false}
    ]'
    ;;
  "test_verify_sign")
    SESSION_ARGS='[
          {"name":"validators","type":{"List":{"ByteArray":20}},"value":["e50b254a5571B59B521e75622C2573067F893132"]}, 
	        {"name":"powers","type":{"List":"U64"},"value":[4294967295]},
          {"name":"valset_nonce","type":"U128","value":"1"},
          {"name":"sigs","type":{"List":"String"},"value":["3c322bc4695d4ce39d2c1262b3beb128c50519937e619ee2ae05ac2ac32d89e116a3e171b0da719a0fc872f391bfae146f089b3fc675df0e4946038f0ae9ac781c"]}
    ]'
    ;;
  "verify_test") ;;
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
