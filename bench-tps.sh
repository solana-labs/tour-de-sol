#!/usr/bin/env bash
set -ex
cd "$(dirname "$0")"

host=$1
if [[ -z $host ]]; then
  host=tds.solana.com
fi

txCount=$2
if [[ -z $txCount ]]; then
  txCount=1000
fi

threadBatchSleepMs=$3
if [[ -z $threadBatchSleepMs ]]; then
  threadBatchSleepMs=250
fi

remote=$4
if [[ -n $remote ]]; then
  exec > solana/client.log
  exec 2>&1
  PATH=$PATH:.cargo/bin/
  killall solana-bench-tps || true

  if [[ $txCount = 0 ]]; then
    exit 0
  fi
else
  scp -o "ConnectTimeout=20" -o "BatchMode=yes" \
    -o "StrictHostKeyChecking=no" -o "UserKnownHostsFile=/dev/null" \
    solana@$host:solana/config/mint-keypair.json .
fi

solana -u http://$host:8899 -k mint-keypair.json balance --lamports

if [[ ! -f bench-tps.json ]]; then
  solana-keygen new -o bench-tps.json
fi

solana -u http://$host:8899 -k mint-keypair.json \
  pay "$(solana-keygen pubkey bench-tps.json)" 10000 SOL
solana -u http://$host:8899 -k bench-tps.json balance

export RUST_LOG=solana=info
solana-bench-tps -i bench-tps.json --tx_count=$txCount \
  -n $host:8001 -N 2 --sustained --thread-batch-sleep-ms=$threadBatchSleepMs
