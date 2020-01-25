#!/usr/bin/env bash
set -e

# Number of lamports to allocate to each validator (2 SOL)
lamports=2000000000

cd "$(dirname "$0")"

# https://github.com/solana-labs/solana/blob/5af7d12756bf59229a9a887e3aab858c76bd9bb9/genesis/src/genesis_accounts.rs#L201-L262
slp_pubkeys="
    27SB7d27xvtBJjgsAV8JBDjQroySmZepiNSepeRbRhe9
    2te46rxywMdCNdkvjumiBBPQoVczJFxhxEaxFavQNqe3
    2tvTYUajoBgeAd66Zhd5Nc2BgKGCgdmasR94fpBokzop
    3QZDKya4AHzsLAuRaMRgejrqW6mETnX88aMSkm7FEE7E
    3Z5XVczCTXeYeFABoeFm1LngC9657kZMVGNFzqFXviHb
    3o43fXxTpndqVsCdMi16WNq6aR9er75P364ZkTHrEQJN
    44e8VyWoyZSE2oYHxMHMedAiHkGJqJgPd3tdt6iKoAFL
    4MHRFcPheQonBf1pUKrBwJAnn2wP9NEZkXYFEFMfFbWV
    4vPqTnfH2ud6hp1yFSFRy9t9xhm8sGDwU4amcZGr2gT7
    4ydifDThiWuVtzV92eNGiznuQAnTZtGkb9b2XQoMGAUn
    54g6LdVubwthdfMKwPqLraDEDAVWNDpN6a3ZGZm2Sbjz
    592eDka2qrXWcszC3NNvViKfEyxvuoAbBgohVt75dWq1
    5JuyDi5HR2CZS39nF43Ws6nhqYWM2fgnZbtf9zRNy52a
    5jTcJaq6gLEao1R5rscvfnUhNt6RXg4JFDCegyEhsJG2
    5n8KCdzqtvTnhkvCrFR7errH6ZUp11kL97r2awXkfzFe
    7ntcPwcaCSpH66ftVZU5oSuWSpvQfN3kfTDaGUHWsc1m
    7sa8uUnjNPJ2dFwrKG2kd1XEiB4ujsJ4wGEWn7CK629K
    7suRNpX7bJsXphHJtBv4ZsLjJZ1dTGeX256pLqJZdEAm
    7v5DXDvYzkgTdFYXYB12ZLKD6z8QfzR53N9hg6XgEQJE
    8LSwP5qYbmuUfKLGwi8XaKJnai9HyZAJTnBovyWebRfd
    8UPb8LMWyoJJC9Aeq9QmTzKZKV2ssov739bTJ14M4ws1
    8oRw7qpj6XgLGXYCDuNoTMCqoJnDd6A8LTpNyqApSfkA
    8wFK4fCAuDoAH1fsgou9yKZPqDMFtJUVoDdkZAAMuhyA
    94eWgQm2k8BXKEWbJP2eScHZeKopXpqkuoVrCofQWBhW
    9J8WcnXxo3ArgEwktfk9tsrf4Rp8h5uPUgnQbQHLvtkd
    AYZS4CFZRi1165mmUqnpDcHbm1NT9zFGPdjG5VDuK79p
    Ah5arzkbkHTMkzUaD5DiCAC1rzxqPgyQDFTnw8Krwz1V
    ArpeD4LKYgza1o6aR5xNTQX3hxeik8URxWNQVpA8wirV
    B21L2hCrdE4SDhRi2fHKohfSUNAhoLeaWfBp1C9HdF6Y
    Bf6JtoLAg9zxAksgZ9gUsa6zZum1UuPWuirY6qKLXXoW
    BrFqUxNY4HstYdiYYZiyDa5KiTrdcfqoBBEky3kqKFgQ
    C8VJytJbZM7KFMXHNUdoF4V7V2QbhkxNs1qYybRoqUEK
    CWfPaZJpy8fc2eU7qe1JNnf4oszQFJU68DZiVJGGy4Z7
    Ccq6zHdtv3DWCP4AccTi4Ya2xPGsEVHSfoPmQ1qffb8H
    ChorusXqjLC2NbiStKR6k9WoD7wu6TVTtFG8qCL5XBVa
    DaqUBvjHtKYiZ6exUhqrcpDqu5ffYB6QWKwXSwdvDVBj
    Daxixc1dFxxLDj85t1CWAsvNXdYq51tDAE51nhPqK9yF
    Dh1DRj5mLYMeJVGvaPZN7F4XjpX6u2dCDXVnUXrE8rwW
    DxLtXrLUrqja3EFjkR4PXNYCuyVtaQnozonCdf3iZk8X
    ETVHRnFkZi7PihPDYibp9fmjfR8P5o7pEs92czku62VV
    EduAgutprA7Vp94ZmTU6WRAmqq7VZAXBqH1GyxjWn12D
    Ez4iUU87ViJLCnmSy1t1Ti3DLoysFXiBseNfnRfoehyY
    FYbyeGqsx8G5mW4p3MfnNEsHaCQQSAmxESf7ct36moGZ
    Fe5sLQAAT7RBT8mcH1AAGCbExJQcYxcwXvp1GjrGbvxs
    FhacRVSACfKcZNAbvbKuj1MunBKxQu2nHu9raJaGsZzG
    FiF184p8DYxnWkBc7WxUh49PccYwvVepmk3nxAnNgGqW
    G47WACh32JUcxyiCna7UYw45tyYSFKQ58yFpUmhmMybm
    GRi3H2M3HxYGAKhz5VrUQipUrAhWj6jTbtjhxiKXHhRj
    GeZ5PrJi9muVCJiJAaFBNGoCEdxGEqTp7L2BmT2WTTy1
    GkNQ9hQM1DoTQy9i4HVzhCjtKh9A6uSx7Z5XTAkqRGhu
    GsEofbB3rzUK78Ee4NRL6AmcPs6FKRCb7JA8tX6LZjHc
    H279DmgqTkTYnEucPdKbvT8wMTGBAuVh787FX2gRT5Bg
    Hac7hGYwbve747fGefaFoank1K1rNmvr5MjtsYvzZ37i
    HavuVVDXXsJqMzPwQ4KcF5kFm2xqjbChhyi1bgGeCQif
    HpzcHxARoR6HtVuZPXWJMLwgusk2UNCan343u6WSQvm2
    Luna1VCsPBE4hghuHaL9UFgimBB3V6u6johyd7hGXBL
    SPC3m89qwxGbqYdg1GuaoeZtgJD2hYoob6c4aKLG1zu
    Smith4JYx2otuFgT2dR83qJSfW8RjBZHPsXPyfZBYBu
    pbAxyqHHPMwgEjv8kmjGxysk9rhNtN7q22eAjReq6Hj
    qzCAEHbjb7AtpTPKaetY47LWNLCxFeHuFozjeXhj1k1
"

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

    declare pubkeyDir=
    for dir in /keybase/public/"$username"/[Ss]olana/; do
      if [[ -d $dir ]]; then
        pubkeyDir=$dir
        break;
      fi
    done

    if [[ -z $pubkeyDir ]]; then
      echo "Warn: $username: $pubkeyDir does not exist"
      continue
    fi

    declare validatorPubkey=
    for file in "$pubkeyDir"validator-*; do
      validatorPubkey=$file

      if [[ $validatorPubkey =~ .*validator-([1-9A-HJ-NP-Za-km-z]+)$ ]]; then
        declare pubkey="${BASH_REMATCH[1]}"

        if [[ $slp_pubkeys =~ $pubkey ]]; then
          echo "Warn: Ignoring SLP pubkey: $pubkey"
          continue
        fi

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
        break
      else
        echo "Warn: $username: invalid validator pubkey: $validatorPubkey"
      fi
      break;
    done

    if [[ -z $validatorPubkey ]]; then
      echo "Warn: $username: no validator pubkey found"
      continue
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
