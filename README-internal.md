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

## Validator Keybase Id Registration
As new stage participants are registered for a given stage, their keybase id should be added to
the file `/stageN/keybase-ids`.  One keybase id per line.

Then prior to the start of the stage, run `./import-keybase-ids.sh` to import
all public keys each validator has published and commit the modifications to
`validator.yml`

## Ledger Rollback Procedure
**Work in progress**

The following steps can be used to perform a ledger rollback if needed:
1. Identity the desired slot height to roll back to
2. Announce to all participants that a rollback is occuring, and request that everybody shut down their validators
3. On the tds.solana.com node, run `solana-ledger-tool ....`
4. Bring the tds.solana.com node back up
5. Bring the other Solana TdS nodes back up
2. Announce to all participants that a rollback has been completed, they should now delete their ledger and restart their validator from a new snapshot
