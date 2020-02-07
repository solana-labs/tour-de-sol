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

who=()
for id_vote in $(solana show-validators | sed -ne "s/^  \([^ ]*\)   *\([^ ]*\) .*/\1=\2/p"); do
  declare id=${id_vote%%=*}
  declare vote=${id_vote##*=}
  declare name=$(grep "$id" ../validators/all-username.yml | head -n1 | cut -d: -f2)

  stake=stake-$id.json
  if [[ -f $stake ]]; then
    who+=($name)
    echo "$name ($id) is already staked"
    continue
  fi

  if ! grep --quiet $id ../validators/all-username.yml; then
    echo "Ignoring unknown validator $id"
    continue
  fi
  echo "Staking $name ($id) (vote account: $vote)"
  who+=($name)

  (
    set -x
    solana-keygen new --no-passphrase --force --silent -o stake.json
    solana create-stake-account stake.json 10000
    solana delegate-stake stake.json $vote
    solana stake-account stake.json
  )
  mv stake.json $stake
done

echo
echo "Staked users: ${#who[@]}"
for user in ${who[@]}; do
  echo "- $user"
done
