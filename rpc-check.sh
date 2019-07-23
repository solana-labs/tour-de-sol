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

url=http://tds.solana.com:8899
if [[ $1 = local ]]; then
  url=http://localhost:8899
fi

set -x
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1, "method":"getLeaderSchedule"}' $url | jq
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1, "method":"getEpochVoteAccounts"}' $url | jq
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1, "method":"getEpochInfo"}' $url | jq
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1, "method":"getClusterNodes"}' $url | jq
if [[ -f validator-keypair.json ]]; then
  solana-wallet --keypair ~/validator-keypair.json --url $url balance
fi
