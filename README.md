# Tour de SOL

## Validator Public Key Registration
In order to obtain your allotment of lamports at the start of a Tour de SOL
stage, you need to publish your validator's identity public key under your
keybase.io account.

**If this registration is not completed by the cut-off time you will not be able to participate.**

1. If you haven't already, generate your validator's identity keypair by running:
     ```bash
     $ solana-keygen new -o ~/validator-keypair.json
     Wrote /Users/<your user name>/validator-keypair.json
     ```
2. The identity public key can now obtained by running:
     ```bash
     $ solana-keygen pubkey ~/validator-keypair.json
     <BASE58_PUBKEY>
     ```
3. Install [Keybase](https://keybase.io/download) on your machine.
3. Create a Solana directory in your public file folder: `mkdir /keybase/public/<keybase id>/solana`
4. Publish your validator's identity public key by creating an empty file in your Keybase public file folder in the following format: `/keybase/public/<KEYBASE_ID>/solana/validator-<BASE58_PUBKEY>`.   For example:
     ```bash
     $ mkdir -p /keybase/public/<KEYBASE_ID>/solana
     $ touch /keybase/public/<KEYBASE_ID>/solana/validator-<BASE58_PUBKEY>
     ```
5. To check your public key was published, ensure you can successfully browse to     `https://keybase.pub/<KEYBASE_ID>/solana/validator-<BASE58_PUBKEY>`


## Connecting Your Validator

Before attempting to connect your validator to the Tour de SOL cluster, be
familiar with connecting a validator to the Public Testnet as described
[here](https://solana-labs.github.io/book/testnet-participation.html).

Ensure the Solana release [v0.16.4](https://github.com/solana-labs/solana/releases/tag/v0.16.4) is installed by running:
```bash
$ SOLANA_RELEASE=v0.16.4 curl -sSf https://raw.githubusercontent.com/solana-labs/solana/v0.16.4/install/solana-install-init.sh | sh -s
```

Your validator identiy keypair will receive an allotment of lamports
in the genesis block that can be used to start your validator node.
*Note that airdrops have been disabled so the `solana-wallet airdrop` command will fail.*

To view your current lamport balance:
```
solana-wallet -k ~/validator-keypair.json -u http://tds.solana.com:8899 balance
```

You can view the other nodes in the cluster using:
```
$ solana-gossip --entrypoint tds.solana.com:8001 spy
```

The easiest way to connect to the Tour de SOL cluster is by running:
```
$ clear-config.sh
$ validator.sh --identity ~/validator-keypair.json --no-airdrop --stake 42 tds.solana.com
```
however this will result in running a validator with a very low stake (42 lamports) and it will likely never be selected as leader.

Increasing your stake can be done by adding the `--stake` argument to
validator.sh followed by the desired stake amount.  *Be mindful that the
validator itself requires lamports to operate, so if you assign your entire
allotment of lamports as stake then your validator will not be able to operate.*

## Monitoring Your Validator
* Use the `solana-wallet balance` command to monitor the earnings as your
  validator is selected as leader and collects transaction fees
* Use the ancillary [Validator
  Metrics](https://solana-labs.github.io/book-edge/testnet-participation.html#validator-metrics)
  distribution to monitor metrics from your validator

## Useful links
* [Solana Book](https://solana-labs.github.io/book/)
* [Network explorer](http://explorer.solana.com/)
* [TdS metrics dashboard](https://metrics.solana.com:3000/d/testnet-beta/testnet-monitor-beta?refresh=1m&from=now-15m&to=now&var-testnet=tds)

## Common Problems

###  Out of disk space in your home directory
By default the ledger will be written into `~/.local/share/solana`.  If you prefer or need to use a different filesystem location run `solana-install init --data-dir /new/data/dir/location/`.  
