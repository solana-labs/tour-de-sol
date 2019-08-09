#!/usr/bin/env bash
set -ex
cd "$(dirname "$0")"

host=$1
if [[ -z $host ]]; then
  host=tds.solana.com
fi

scp -o "ConnectTimeout=20" -o "BatchMode=yes" \
  -o "StrictHostKeyChecking=no" -o "UserKnownHostsFile=/dev/null" \
  solana@$host:solana/config/mint-keypair.json .

solana-wallet -u http://$host:8899 -k mint-keypair.json balance

if [[ ! -f bench-tps.json ]]; then
  solana-keygen new -o bench-tps.json
fi

solana-wallet -u http://$host:8899 -k mint-keypair.json \
  pay "$(solana-keygen pubkey bench-tps.json)" 100000000
solana-wallet -u http://$host:8899 -k bench-tps.json balance

export RUST_LOG=solana=info
solana-bench-tps -n $host:8001 -i bench-tps.json -N 2 --tx_count=1000 --thread-batch-sleep-ms=100
