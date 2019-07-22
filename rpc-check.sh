#!/usr/bin/env bash

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
