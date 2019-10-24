# Internal
Information useful for running a TdS stage.

## Configuring/Running the Tour de Sol Cluster
Buildkite jobs are used to set up and tear down the Solana nodes on the TdS cluster, and configure some genesis block settings.
Most of the relevant settings can be set at environment variables within a buildkite job, so we can change and re-deploy rapidly without needing to touch CI script code.

### Enable TdS cluster
Run a job of the TdS-enable pipeline.  Click 'New Build' and run with default settings

https://buildkite.com/solana-labs/tds-enable

This only needs to be run once per CI instance, or after the TdS cluster has been disabled with the TdS-delete-disable pipeline.

### Create and start a new TdS cluster
To create a new cluster, use the TdS-create-and-start buildkite job:

https://buildkite.com/solana-labs/tds-create-and-start

The pipeline will pull the tip of the v0.16 branch of code for scripts and binaries by default.
All of the default configuration settings for the cluster can be found in the pipeline settings:

https://buildkite.com/solana-labs/tds-create-and-start/settings

Any of the above values can be overwritten for a particular build, by using `[key]=[value]` syntax (do not use double quotes for the value here) under Environment Variables when you click 'New Build'.

The `TDS_ZONES`, `TDS_NODE_COUNT` and `TDS_CLIENT_COUNT` must have a valid value if the default is not used.  All other variables may set the value to `skip` to disable the given configuration and the system default behavior will be used.

NOTE: Using the `STAKE_INTERNAL_NODES` setting (including the default value in this pipeline) disables airdrops for the cluster.

Example:  Enter the following in the Environment Variables box for a New Build to have only CPU-only validator node in a single region.  No clients and no GPUs.
```bash
  TDS_ZONES=us-west1-a
  TDS_CLIENT_COUNT=0
  ENABLE_GPU=skip
```

Example:  If you want to run the cluster on the tip of master instead of the v0.16 branch, in the New Build window, set the Branch field to `master` and add the following to the Environment Variables:
```bash
TESTNET_TAG=edge
```

Example:  To enable airdrops, add the following in Environment Variables:
```bash
STAKE_INTERNAL_NODES=skip
```

### Restart network software on existing nodes
To restart the binary software on the nodes without deleting and re-creating the instances, use the TdS-restart pipeline

https://buildkite.com/solana-labs/tds-restart

The number of nodes, GPUs, and the zones cannot be changed from this pipeline but new settings for the genesis block can be provided.  The following settings can be changed when restarting the cluster/ledger:
```bash
  HASHES_PER_TICK: "auto"
  STAKE_INTERNAL_NODES: "1000000000000"
  EXTERNAL_ACCOUNTS_FILE_URL: "https://raw.githubusercontent.com/solana-labs/tour-de-sol/master/stage1/validator.yml"
  LAMPORTS: "8589934592000000000"
  ```

## Validator Keybase Username Registration
As new stage participants are registered for a given stage, their keybase username should be added to
one of the keybase-username files, one keybase username per line:
* `validators/keybase-usernames.internal` - Solana internal
* `validators/keybase-usernames.us` - us-based validators
* `validators/keybase-usernames.earth` - earth-based validators, excluding us.

Then prior to the start of the stage, run `./import-keybase-usernames.sh` to import
all public keys each validator has published and commit the modifications to
`validators/*.yml`

### Running bench-tps on the cluster
```bash
$ ./bench-tps.sh
```

## Attaching to the TdS cluster
Fetching the TdS cluster configuration can be accomplished with:
```bash
$ net/gce.sh config -p tds-solana-com -z us-west1-a -z us-central1-a -z europe-west4-a
```
at which point all the normal `net/` functionality becomes available (such as `net/ssh.sh`).   Also `net/net.sh logs` can be used to collect logs off the nodes

## Ledger Rollback Procedure
**Work in progress**

The following steps can be used to perform a ledger rollback if needed:
1. Identify the desired slot height to roll back to
2. Announce to all participants that a rollback is occuring, and request that everybody shut down their validators
3. Stop the Solana TdS nodes: `./net stop`
3. On the tds.solana.com bootstrap-leader node, run the following steps to generate a rollback list
```bash
$ solana-ledger-tool --ledger ${path_to_ledger} list-roots --max-height ${rollback_slot_height} --slot-list ./rollback.txt
$ solana-ledger-tool --ledger ${path_to_ledger} prune --slot-list rollback.txt
# The output should look something like this
Prune at slot 5000 hash "HRQnaDnSoaeM5xQKxjKYbU53ZFhTYtjBS7HWyG3Q1JUq"
```
4. Bring the Solana TdS nodes back up with `./net start --no-deploy --no-snapshot --skip-ledger-verify -r`
2. Announce to all participants that a rollback has been completed, they should now delete their ledger and restart their validator from a new snapshot

## TPS Ramp-up Procedure

#### Directions
1. [Fetch the TdS cluster configuration](#attaching-to-the-tds-cluster)
1. Set bash vars for the network
```bash
$ eval $(net/gce.sh info --eval)
```
1. Snag the mint keypair from the bootstrap leader
```bash
$ net/scp.sh solana@"$NET_VALIDATOR0_IP":solana/config/mint-keypair.json .
```
1. Optionally set SLACK env vars to be notified of progress
```bash
export SLACK_TOKEN=
export SLACK_CHANNEL_ID=
```
1. Start the ramp-up TPS tool
```bash
$ cargo run -p solana-ramp-tps -- -n $NET_VALIDATOR0_IP \
  --net-dir <solana/net> \
  --round-minutes 15 \
  --tps-baseline 5000 \
  --tps-increment 5000 \
  --mint-keypair-path <mint_keypair.json>
```

#### Recovery
If the tool fails, it may be possible to recover and pickup where it last
left off. The only unsupported scenario is when the tool fails in the
middle of awarding stake to the surviving validators.

- If the tool failed during bench-tps, recovery is simple. Simply start
the tool at the `round` number which failed.
- If the tool fails during stake warmup, specify both the TPS `round` number
as well as the epoch when the stake started activating (`stake-activation-epoch`).

```bash
$ cargo run -p solana-ramp-tps -- -n $NET_VALIDATOR0_IP \
  --net-dir <solana/net> \
  --round <START ROUND> \
  --round-minutes 15 \
  --tps-baseline 5000 \
  --tps-increment 5000 \
  --stake-activation-epoch <LAST STAKE ACTIVATION EPOCH> \
  --mint-keypair-path <mint_keypair.json>
```

#### Overview
The ramp up tool will be following this process:

1. Download the genesis block
1. Wait for warm up epochs to pass
1. Start ramp up cycle
  1. Wait for validator stakes to warm up
  1. Run solana-bench-tps on clients
  1. Sleep until the round is finished
  1. Stop solana-bench-tps
  1. Fetch top performing validators
  1. Gift stake to the top validators
  1. Double gift and increment TPS
