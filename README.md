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
3. Create a Solana directory in your public file folder: `mkdir /keybase/public/<KEYBASE_USERNAME>/solana`
4. Publish your validator's identity public key by creating an empty file in your Keybase public file folder in the following format: `/keybase/public/<KEYBASE_USERNAME>/solana/validator-<BASE58_PUBKEY>`.   For example:
     ```bash
     $ mkdir -p /keybase/public/<KEYBASE_USERNAME>/solana
     $ touch /keybase/public/<KEYBASE_USERNAME>/solana/validator-<BASE58_PUBKEY>
     ```
5. To check your public key was published, ensure you can successfully browse to     `https://keybase.pub/<KEYBASE_USERNAME>/solana/validator-<BASE58_PUBKEY>`


## Connecting Your Validator

Before attempting to connect your validator to the Tour de SOL cluster, be
familiar with connecting a validator to the Public Testnet as described
[here](https://solana-labs.github.io/book-edge/running-validator.html).

Ensure the Solana release [v0.18.0](https://github.com/solana-labs/solana/releases/tag/v0.18.0) is installed by running:
```bash
$ curl -sSf https://raw.githubusercontent.com/solana-labs/solana/v0.17.1/install/solana-install-init.sh | sh -s - 0.18.0
```

Configure solana for your validator identity and Tour de SOL:
```bash
$ solana set --url http://tds.solana.com:8899 --keypair ~/validator-keypair.json
```

Your validator identity keypair will receive an allotment of lamports
in the genesis block that can be used to start your validator node.
*Note that airdrops have been disabled so the `solana airdrop` command will fail.*

To view your current lamport balance:
```
$ solana balance
```

You can view the other nodes in the cluster using:
```
$ solana-gossip --entrypoint tds.solana.com:8001 spy
```

The `ping` commmand can be used to check that the cluster is able to process transactions:
```
$ solana ping
```

Create your vote account:
```bash
$ solana-keygen new -o ~/validator-vote-keypair.json
$ solana create-vote-account ~/validator-vote-keypair.json ~/validator-keypair.json 1

```

Connect to the Tour de SOL cluster by running:
```bash
$ export SOLANA_METRICS_CONFIG="host=https://tds-metrics.solana.com:8086,db=tds,u=tds_writer,p=dry_run"
$ solana-validator --identity ~/validator-keypair.json --voting-keypair ~/validator-vote-keypair.json \
    --ledger ~/validator-ledger --rpc-port 8899 --entrypoint tds.solana.com:8001 \
    --limit-ledger-size
```

**By default your validator will have no stake.**  
Once your validator is caught up to the tip of the cluster, you can add stake by running:
```bash
$ solana-keygen new -o ~/validator-stake-keypair.json
$ solana delegate-stake ~/validator-stake-keypair.json ~/validator-vote-keypair.json 8589934592
```

More information about staking can be found at https://solana-labs.github.io/book-edge/validator-stake.html

## Publishing Information About Your Validator
See https://solana-labs.github.io/book-edge/validator-info.html for background,
to operate `solana-validator-info` on the TdS cluster you need to include the
`-u http://tds.solana.com:8899` argument:

Example publish command:
```bash
$ solana-validator-info publish -u http://tds.solana.com:8899 ~/validator-keypair.json ...
```

Example query command:
```bash
$ solana-validator-info get -u http://tds.solana.com:8899
Validator info from 8WdJvDz6obhADdxpGCiJKZsDYwTLNEDFizayqziDc9ah
  Validator pubkey: 6dMH3u76qZ7XG4bVboVRnBHR2FfrxEqTTTyj4xmyDMWo
  Info: {"keybaseUsername":"mvines","name":"mvines","website":"https://solana.com"}
```

## Monitoring Your Validator
* Run `solana get-slot` to track the progress of your validator as it catches up with the cluster after you first connect.
* Use the `solana balance` command to monitor the earnings as your
  validator is selected as leader and collects transaction fees
* Run [rpc-check.sh](https://github.com/solana-labs/tour-de-sol/blob/master/rpc-check.sh) periodically

## Useful links
* [Solana Book](https://solana-labs.github.io/book-edge/)
* [Network explorer](http://explorer.solana.com/)
* [TdS metrics dashboard](https://metrics.solana.com:3000/d/testnet-edge/testnet-monitor-edge?refresh=1m&from=now-15m&to=now&var-testnet=tds&orgId=2&var-datasource=TdS%20Metrics%20(read-only))

## Common Problems

### ...
