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

for id_vote in $(solana show-validators | sed -ne "s/^  \([^ ]*\)   *\([^ ]*\) .*/\1=\2/p"); do
  declare id=${id_vote%%=*}
  declare vote=${id_vote##*=}

  stake=stake-$id.json
  if [[ -f $stake ]]; then
    echo "$vote (id: $id) is already staked"
    continue
  fi

  if ! grep --quiet $id ../validators/all-username.yml; then
    echo "Ignoring unknown validator $id"
    continue
  fi

  echo "Staking $id (vote account: $vote)"

  (
    set -x
    solana-keygen new --no-passphrase --force --silent -o stake.json
    solana create-stake-account stake.json 10000
    solana delegate-stake stake.json $vote
    solana stake-account stake.json
  )
  mv stake.json $stake
done

