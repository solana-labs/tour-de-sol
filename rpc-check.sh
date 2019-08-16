#!/usr/bin/env bash
#
# Outputs some useful information about the TdS cluster.
#
# Example:
#   $ ./rpc-check.sh        # <-- query the TdS cluster entrypoint
#   $ ./rpc-check.sh local  # <-- query your local node (which should match what the TdS cluster entrypoint returned)
#
# Requires: https://stedolan.github.io/jq/
#

here=$(dirname "$0")
source $here/get-url.sh

set -x
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1, "method":"getLeaderSchedule"}' $url | jq
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1, "method":"getVoteAccounts"}' $url | jq
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1, "method":"getClusterNodes"}' $url | jq
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1, "method":"getEpochInfo"}' $url | jq
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1, "method":"getSlot"}' $url | jq
if [[ -f ~/validator-keypair.json ]]; then
  solana-wallet --keypair ~/validator-keypair.json --url $url balance
fi
