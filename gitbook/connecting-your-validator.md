# Connecting Your Validator

Before attempting to connect your validator to the Tour de SOL cluster, be familiar with connecting a validator to the Public Testnet as described [here](https://solana-labs.github.io/book-edge/running-validator.html).

Ensure the Solana release [v0.18.0](https://github.com/solana-labs/solana/releases/tag/v0.18.0) is installed by running:

```bash
$ curl -sSf https://raw.githubusercontent.com/solana-labs/solana/v0.17.1/install/solana-install-init.sh | sh -s - 0.18.0
```

Configure solana for your validator identity and Tour de SOL:

```bash
$ solana set --url http://tds.solana.com:8899 --keypair ~/validator-keypair.json
```

Your validator identity keypair will receive an allotment of SOL in the genesis block that can be used to start your validator node. _Note that airdrops have been disabled so the `solana airdrop` command will fail._

To view your current balance:

```text
$ solana balance
```

Or to see in finer detail:

```text
$ solana balance --lamports
```

You can view the other nodes in the cluster using:

```text
$ solana-gossip --entrypoint tds.solana.com:8001 spy
```

The `ping` commmand can be used to check that the cluster is able to process transactions:

```text
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
$ solana delegate-stake ~/validator-stake-keypair.json ~/validator-vote-keypair.json 0.5
```

More information about staking can be found at [https://solana-labs.github.io/book-edge/validator-stake.html](https://solana-labs.github.io/book-edge/validator-stake.html)

## 



## 

