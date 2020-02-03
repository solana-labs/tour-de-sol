#!/usr/bin/env bash
#
# Delegate 10,000 SOL to all current validators.  Idempotent
#

set -e

cd "$(dirname "$0")"

(
  set -x
  solana balance
)

for vote in $(solana show-validators | sed -ne "s/^  \([^ ]*\)   *\([^ ]*\)      .*/\2/p"); do
  stake=stake-$vote.json
  if [[ -f $stake ]]; then
    echo "$vote is already staked"
    continue
  fi

  (
    set -x
    solana-keygen new --no-passphrase --force --silent -o stake.json
    solana create-stake-account stake.json 10000
    solana delegate-stake stake.json $vote
    solana stake-account stake.json
  )
  mv stake.json $stake
done

