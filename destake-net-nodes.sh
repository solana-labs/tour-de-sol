#!/usr/bin/env bash
#
# Deactivates the default stake from net/ nodes, then grants each node with a new
# stake of 1 SOL
#

netdir=$1

if [[ -z $netdir ]]; then
  echo "Usage: $0 net-dir"
  exit 1
fi

if [[ ! -r "$netdir"/gce.sh ]]; then
  echo "Error: invalid netdir: $netdir"
  exit 1
fi

eval $("$netdir"/gce.sh info --eval)

scp="$netdir"/scp.sh
solana="solana --url http://$NET_VALIDATOR0_IP:8899"

rm -rf .destake
mkdir .destake
cd .destake

echo Fetching bootstrap leader keys
$scp solana@"$NET_VALIDATOR0_IP":~/solana/config/faucet-keypair.json .
$scp solana@"$NET_VALIDATOR0_IP":~/solana/config/bootstrap-leader/stake-keypair.json 0-stake-keypair.json
$scp solana@"$NET_VALIDATOR0_IP":~/solana/config/bootstrap-leader/vote-keypair.json 0-vote-keypair.json
$scp solana@"$NET_VALIDATOR0_IP":~/solana/config/bootstrap-leader/identity-keypair.json 0-identity-keypair.json
$scp solana@"$NET_VALIDATOR0_IP":~/solana/config/tds-stake-keypair.json 0-tds-stake-keypair.json

for i in $(seq 1 $((NET_NUM_VALIDATORS - 1))); do
  v="NET_VALIDATOR${i}_IP"
  echo "Fetching $v keys"
  $scp solana@"${!v}":~/solana/config/validator/stake-keypair.json $i-stake-keypair.json
  $scp solana@"${!v}":~/solana/config/validator/vote-keypair.json $i-vote-keypair.json
  $scp solana@"${!v}":~/solana/config/validator-identity.json $i-identity-keypair.json
  $scp solana@"${!v}":~/solana/config/tds-stake-keypair.json $i-tds-stake-keypair.json
done

ls -l
$solana balance

for i in $(seq 0 $((NET_NUM_VALIDATORS - 1))); do
  v="NET_VALIDATOR${i}_IP"
  if [[ -f $i-tds-stake-keypair.json ]]; then
    echo "Validator $i already re-staked"
  else
    (
      set -x
      solana-keygen new --no-passphrase -f-o $i-tds-stake-keypair.json
      $solana --keypair $i-identity-keypair.json deactivate-stake $i-stake-keypair.json
      $solana --keypair faucet-keypair.json create-stake-account $i-tds-stake-keypair.json 1 SOL
      $solana --keypair faucet-keypair.json delegate-stake --keypair faucet-keypair.json $i-tds-stake-keypair.json $i-vote-keypair.json
      $scp $i-tds-stake-keypair.json solana@"${!v}":~/solana/config/tds-stake-keypair.json
    )
  fi
done

exit 0
