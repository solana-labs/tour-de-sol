#!/usr/bin/env bash
#
# Outputs all stake accounts

# Requires: https://stedolan.github.io/jq/
#

here=$(dirname "$0")
source $here/get-url.sh

for pubkey in $(
    curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0", "id":1, "method":"getProgramAccounts", "params":["Stake11111111111111111111111111111111111111"]}' $url | jq -c '.result[][0]'
  ); do
  (
    set -x
    solana --url $url show-stake-account ${pubkey//\"}
  )
done

