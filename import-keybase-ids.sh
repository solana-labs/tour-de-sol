#!/usr/bin/env bash
set -e

# Number of lamports to allocate to each validator
lamports=17179869184

stage=$1
[[ -n $stage ]] || {
  echo "Usage: $0 <stage-number>"
  exit 1
}

cd "$(dirname "$0")"
[[ -d stage"$stage"/ ]] || {
  echo "stage$stage/ doesn't exist"
  exit 1
}

cd stage"$stage"/

echo > validator.yml
shopt -s nullglob
for id in $(cat keybase-ids); do
  echo "Processing $id..."
  declare pubkeyDir=/keybase/public/"$id"/solana/
  if [[ ! -d "$pubkeyDir" ]]; then
    echo "Warn: $id: $pubkeyDir does not exist"
    continue
  fi

  declare validatorPubkey=
  for file in "$pubkeyDir"validator-*; do
    validatorPubkey=$file
    break;
  done

  if [[ -z $validatorPubkey ]]; then
    echo "Warn: $id: no validator pubkey found"
    continue
  fi

  if [[ $validatorPubkey =~ .*validator-([1-9A-HJ-NP-Za-km-z]+)$ ]]; then
    declare pubkey="${BASH_REMATCH[1]}"
    echo "$pubkey registered"
    echo "$pubkey: $lamports" >> validator.yml
  else
    echo "Warn: $id: invalid validator pubkey: $validatorPubkey"
  fi
done

echo
echo Wrote validator.yml
