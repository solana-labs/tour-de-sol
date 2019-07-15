# Tour de SOL

## Validator Public Key Registration
In order to obtain your allotment of lamports at the start of a Tour de SOL stage, you need to publish your validator's identity public key under your keybase.io account.

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
```bash
  TDS_ZONES: "us-west1-a,us-central1-a,europe-west4-a"
  TDS_NODE_COUNT: "3"
  TDS_CLIENT_COUNT: "1"
  ENABLE_GPU: "--machine-type n1-standard-16 --accelerator count=2,type=nvidia-tesla-v100"
  HASHES_PER_TICK: "auto"
  STAKE_INTERNAL_NODES: "1000000000000"
  EXTERNAL_ACCOUNTS_FILE_URL: "https://raw.githubusercontent.com/solana-labs/tour-de-sol/master/stage1/validator.yml"
  LAMPORTS: "8589934592000000000"
  ADDITIONAL_DISK_SIZE_GB: "32000"
```

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

You can also specify different binaries to run on the nodes by setting the `TESTNET_TAG` value in Environment Variables.
