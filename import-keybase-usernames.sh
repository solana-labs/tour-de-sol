#!/usr/bin/env bash
set -e

# Number of lamports to allocate to each validator (500 SOL)
lamports=8589934592000

cd "$(dirname "$0")"

rm -f validators/all.md
shopt -s nullglob
for keybase_file in validators/keybase-usernames.*; do
  section=${keybase_file##*.}
  echo "## $section" >> validators/all.md
  pubkey_yml=${keybase_file/keybase-usernames./}-pubkey.yml
  rm -f $pubkey_yml
  touch $pubkey_yml
  username_yml=${keybase_file/keybase-usernames./}-username.yml
  rm -f $username_yml
  touch $username_yml
  for username in $(cat "$keybase_file"); do
    echo "Processing $username..."
    declare pubkeyDir=/keybase/public/"$username"/solana/
    if [[ ! -d "$pubkeyDir" ]]; then
      echo "Warn: $username: $pubkeyDir does not exist"
      continue
    fi

    declare validatorPubkey=
    for file in "$pubkeyDir"validator-*; do
      validatorPubkey=$file
      break;
    done

    if [[ -z $validatorPubkey ]]; then
      echo "Warn: $username: no validator pubkey found"
      continue
    fi

    if [[ $validatorPubkey =~ .*validator-([1-9A-HJ-NP-Za-km-z]+)$ ]]; then
      declare pubkey="${BASH_REMATCH[1]}"
      echo "$pubkey registered"
      cat >> $pubkey_yml <<EOF
$pubkey:
  balance: $lamports
  owner: 11111111111111111111111111111111
  data:
  executable: false
EOF

      echo "$pubkey: $username" >> $username_yml
      echo "1. [$username](https://keybase.io/$username): \`$pubkey\`" >> validators/all.md
    else
      echo "Warn: $username: invalid validator pubkey: $validatorPubkey"
    fi
  done
  echo Wrote $pubkey_yml $username_yml
done

echo
pubkey_yml=validators/all-pubkey.yml
rm -f $pubkey_yml
cat validators/*-pubkey.yml > $pubkey_yml
echo Wrote $pubkey_yml

username_yml=validators/all-username.yml
rm -f $username_yml
cat validators/*-username.yml > $username_yml
echo Wrote $username_yml
