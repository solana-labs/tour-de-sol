# Participation FAQs

## 1.1 What are the requirements to run a Solana validator?

* **Hardware**
  * See [suggested hardware configuration here](https://solana-labs.github.io/book/testnet-participation.html#machine-requirements).
  * CPU Recommendations
    * We recommend a CPU with the highest number of cores as possible. AMD Threadripper or Intel Server \(Xeon\) CPUs are fine. 
    * We recommend AMD Threadripper as you get a larger number of cores for parallelization compared to Intel. 
    * Threadripper also has a cost-per-core advantage and a greater number of PCIe lanes compared to the equivalent Intel part. PoH \(Proof of History\) is based on sha256 and Threadripper also supports sha256 hardware instructions.
  * SSD size and I/O style \(SATA vs NVMe/M.2\)      for a validator
    * Minimum example - Samsung 860 Evo 2TB
    * Mid-range example - Samsung 860 Evo 4TB
    * High-end example - Samsung 860 Evo 4TB
  * GPUs
    * **Validator** nodes will be required to run with GPUs starting at Stage 1 of Tour de SOL. Without GPUs, a validator will not be able to catch up to the ledger once the network is launched. GPUs are NOT required for validators during Stage 0/Dry Runs of Tour de SOL.
    * What kind of GPU?
      * We recommend Nvidia 2080Ti or 1080Ti series consumer GPU or Tesla series server GPUs.
      * We do not currently support OpenCL and therefore do not support AMD GPUs.  We have a bounty out for someone to port us to OpenCL. Interested?
  * Power Consumption
    * Approximate power consumption for a validator node running an AMD Threadripper 2950W and 2x 2080Ti GPUs is 800-1000W.
* **Software**
  * We build and run on Ubuntu 18.04.  Some users have had trouble when running on Ubuntu 16.04
  * See Section 4 below for the current Solana software release.

## 1.2 What are the steps to create a Solana validator?

To create a Solana validator, complete the following steps, using the instructions that follow this section.

* Create your validator public key.
* Install the Solana software.
* Create and configure your validator.
* Confirm the Solana network is running.
* Connect your validator to the Solana network.
* Execute core validator functions.

## 1.3 How do I create my validator public key? \([Source](https://github.com/solana-labs/tour-de-sol)\)

In order to participate in any Tour de SOL dry-runs or stages, you need to register for the Tour de SOL.

See Registration info on [Registration FAQs.](validator-registration-and-rewards-faq.md)

If this registration is not completed by the TdS Stage 1 cut-off time, you will not be able to participate in Stage 1 or subsequent phases.

### If you haven't already, generate your validator's identity keypair by running:

`$ solana-keygen new -o ~/validator-keypair.json  
Wrote /Users/<your user name>/validator-keypair.json`

The identity keypair public key can now viewed by running:

`$ solana-keygen pubkey ~/validator-keypair.json  
<BASE58_PUBKEY>`

{% hint style="info" %}
Note: The "validator-keypair.json” file is also your \(ed25519\) private key.
{% endhint %}

Your validator identity keypair uniquely identifies your validator within the network. **It is crucial to back-up this information.**

If you don’t back up this information, you WILL NOT BE ABLE TO RECOVER YOUR VALIDATOR, if you lose access to it. If this happens, YOU WILL LOSE YOUR ALLOCATION OF LAMPORTS TOO.

To back-up your validator identify keypair, **back-up your "validator-keypair.json” file to a secure location.**

Your validator identity keypair will receive an allotment of lamports in the TdS genesis block that can be used to start your validator node. Note that airdrops have been disabled so the `solana airdrop` command will fail.

### Sign your Solana pubkey with a Keybase account

You must sign your Solana pubkey with a Keybase.io account. The following instructions describe how to do that by installing Keybase on your server. 

* [Install Keybase](https://keybase.io/download) on your server
  * Login to your Keybase account on your server. Create a Keybase account first if you don’t already have one. Here’s a [list of basic Keybase CLI commands](https://keybase.io/docs/command_line/basics).
  * Create a Solana directory in your public file folder:
  * `$ mkdir /keybase/public/<KEYBASE_USERNAME>/solana`
  * Publish your validator's identity public key by creating an empty file in your Keybase public file folder in the following format: `/keybase/public/<KEYBASE_USERNAME>/solana/validator-<BASE58_PUBKEY>`. For example: 
  * `$ touch /keybase/public/<KEYBASE_USERNAME>/solana/validator- <BASE58_PUBKEY>`
  * Confirm your pubkey was published to your Keybase profile.

To check your public key was published, ensure you can successfully browse to:

[`https://keybase.pub/`](https://keybase.pub/)`<KEYBASE_USERNAME>/solana/validator-<BASE58_PUBKEY>`

## 1.4 How do I install the Solana software? \([Source](https://github.com/solana-labs/tour-de-sol)\)

You can confirm the version running on the cluster entrypoint by running:

`$ curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0","id":1,"method":"getVersion"}' tds.solana.com:8899  
{"jsonrpc":"2.0","result":{"solana-core":"0.18.0"},"id":1}`

{% hint style="info" %}
Note: Before installing the Solana software, make sure you’ve generated your validator identity keypair, as described above in Step 2.
{% endhint %}

Install the specified Solana release by running:

`$ curl -sSf https://raw.githubusercontent.com/solana-labs/solana/v0.18.0/install/solana-install-init.sh | sh -s - 0.18.0`

The following output indicates a successful update: 

`looking for latest release  
downloading v0.18.0 installer  
Configuration: /home/solana/.config/solana/install/config.yml  
Active release directory: /home/solana/.local/share/solana/install/active_release  
* Release version: 0.18.0  
* Release URL: https://github.com/solana-labs/solana/releases/download/v0.18.0/solana-release-x86_64-unknown-linux-gnu.tar.bz2  
Update successful`

## 1.5 How do I create and configure my Solana validator?

The validator identity keypair you created above identifies your validator. Your validator identity keypair associated with the public key you registered for the TdS will receive an allotment of lamports in the genesis block that can be used to start your validator node. 

\(Note: Airdrops have been disabled for the Tour de SOL, so the `solana airdrop` command will fail.\)

First, configure the Solana CLI to use the TdS network and your validator identity keypair for all following commands:

`$ solana set --keypair ~/validator-keypair.json --url` [`http://tds.solana.com:8899`](http://tds.solana.com:8899)\`\`

You should see the following output:

`Wallet Config Updated: /home/solana/.config/solana/wallet/config.yml  
* url: http://tds.solana.com:8899  
* keypair: /home/solana/validator-keypair.json`

You can see the wallet configuration at any time by running:

`$ solana get`

To view your current lamport balance:

`$ solana balance`

If your validator identity keypair is created and the above command shows a non-zero balance, your validator is created and configured for TdS.

Your starting balance for both dry-runs and Stage 1 will be 1000 SOL. It will be displayed as 17179869184000 lamports. Read more about the [difference between SOL and lamports here](https://solana-labs.github.io/book/introduction.html?highlight=lamport#what-are-sols).

The next steps are to:

* Confirm the network is running 
* Connect your validator to the network and 
* Delegate your lamports to you validator

## 1.6 How can I tell if the network is running?

Before you connect your validator to the Solana network, confirm the network is running. To do this, view the existing nodes in the network using:

`$ solana-gossip --entrypoint tds.solana.com:8001 spy`

If you see more than 1 node listed in the output of the above command, the network is running.

You can also run the following command to confirm the network is operational:

`$ solana ping`  
  
This command sends a tiny transaction every 2 seconds and reports how long it takes to confirm it.

## 1.7 How do I connect my validator to the Solana network?

Once you’ve confirmed the network is running, it’s time to connect your validator to the network. 

If you haven’t already done so, create a vote-account keypair and create the vote account on the network. If you have completed this step, you should see the “validator-vote-keypair.json” in your Solana runtime directory.

`$ solana-keygen new -o ~/validator-vote-keypair.json`

`$ solana create-vote-account ~/validator-vote-keypair.json ~/validator-keypair.json 1`

Also, make sure you delete any previous solana ledgers \(or specify a different --ledger directory in the command below\):

`$ rm -rf ~/validator-ledger`

Connect to the Tour de SOL network by running:

`$ export SOLANA_METRICS_CONFIG="host=https://tds-metrics.solana.com:8086,db=tds,u=tds_writer,p=dry_run"`

`$ solana-validator --identity ~/validator-keypair.json --voting-keypair ~/validator-vote-keypair.json --ledger ~/validator-ledger --rpc-port 8899 --limit-ledger-size --entrypoint tds.solana.com:8001`

Confirm your validator connected to the network by running:

`$ solana-gossip --entrypoint tds.solana.com:8001 spy`

This command will display all the nodes that are visible to the TdS network’s entrypoint.   If your validator is connected, its public key and IP address will appear in the list.

## 1.8 Now that my validator is connected to the Solana network, how do I execute core validator functions?

Now that your validator is connected, you need to:

### Monitor your validator to confirm it catches up to the latest network slot

{% hint style="info" %}
This is also referred to as your validator “catching-up to the tip of the network”.
{% endhint %}

After your validator is connected, it may take some time to catch up with the network. Use the `get-slot` wallet command to view the current slot that the network is processing: 

`$ solana get-slot`

The current slot that your validator is processing can then been seen with:

`$ solana --url http://127.0.0.1:8899 get-slot`

To see both values at once, run:

`$ echo "me: $(solana --url http://127.0.0.1:8899 get-slot | grep '^[0-9]\+$'), cluster: $(solana --url http://tds.solana.com:8899 get-slot | grep '^[0-9]\+$')"`

Your validator is caught-up when your validator’s current slot matches the current slot the network is processing.

Until your validator has caught up, it will not be able to vote successfully and stake cannot be delegated to it.

Also if you find the network's slot advancing faster than yours, you will likely never catch up.  This typically implies some kind of networking issue between your validator and the rest of the network.

We are working to publish a troubleshooting checklist. Until that time, please contact a Solana team member in the [\#tourdesol-stage0](https://discord.gg/Xf8tES) Discord channel for troubleshooting assistance.

### Delegate stake to your validator

In order for your validator to start voting, you have to delegate stake, in the form of lamports, to it. 

Remember: 

* The validator identity keypair you registered and are using with your validator, by following the above instructions, should already have received an allotment of lamports.
* Your validator has to be caught up to the network, as explained above, before you can delegate stake to it.

You can now delegate some of those lamports to your validator. The current recommendation is to delegate 0.5 SOL to your validator.

_Don’t delegate your entire balance,_ as the validator needs lamports to operate. 

If you haven’t already done so, create a staking keypair. If you have completed this step, you should see the “validator-stake-keypair..json” in your Solana runtime directory.

`$ solana-keygen new -o ~/validator-stake-keypair.json`

Then, run this command, for example, to delegate 8589934592 lamports :

`$ solana delegate-stake ~/validator-stake-keypair.json ~/validator-vote-keypair.json 8589934592`

You should see this output, that lists the transaction hash:

`solana delegate-stake ~/validator-stake-keypair.json ~/validator-vote-keypair.json 8589934592  
Using RPC Endpoint: http://tds.solana.com:8899  
23iejA34QpzYjKD6A7NqMs5xyEvvSBzP4AfW953Z4KrsBP8M12NDfy7EHpt9ALnCyD2wFzK3L8HxZ4LZjJGgMEY2`

At the end of each slot, a validator is expected to send a vote transaction. These vote transactions are paid for by lamports from a validator's identity account. 

This is a normal transaction so the standard transaction fee will apply. The transaction fee range is defined by the genesis block. The actual fee will fluctuate based on transaction load. You can determine the current fee via the [RPC API “getRecentBlockhash”](https://solana-labs.github.io/book-edge/jsonrpc-api.html#getrecentblockhash) before submitting a transaction.

Learn more about [transaction fees here](https://solana-labs.github.io/book-edge/transaction-fees.html).

### Monitor your validator during its “warm up” period; Confirm your validator becomes a “[leader](https://solana-labs.github.io/book/terminology.html#leader)”

* View your vote account:`$ solana show-vote-account ~/validator-vote-keypair.json`
* This displays the current state of all the votes the validator has submitted to the network.
* View your stake account, the delegation preference and details of your stake:`$ solana show-stake-account ~/validator-stake-keypair.json`

### [RPC Commands](https://solana-labs.github.io/book-edge/jsonrpc-api.html)

`getEpochInfo`

[An epoch](https://solana-labs.github.io/book/terminology.html#epoch) is the time, i.e. number of [slots](https://solana-labs.github.io/book/terminology.html?highlight=epoch#slot), for which a [leader schedule](https://solana-labs.github.io/book/terminology.html?highlight=epoch#leader-schedule) is valid. This will tell you what the current epoch is and how far into it the cluster is.

`getVoteAccounts` 

This will tell you how much active stake your validator currently has. A % of the validator's stake is activated on an epoch boundary. You can learn more about staking on Solana [here](https://solana-labs.github.io/book-edge/stake-delegation-and-rewards.html).

`getLeaderSchedule`

At any given moment, the network expects only one validator to produce ledger entries. The [validator currently selected to produce ledger entries](https://solana-labs.github.io/book/leader-rotation.html?highlight=leader#leader-rotation) is called the “leader”.

This will return the complete leader schedule \(on a slot-by-slot basis\) for the current epoch. If you validator is scheduled to be leader based on its currently activated stake, the identity pubkey will show up 1 or more times here.

## 1.9 Where can I submit bugs and feedback requests?

Please submit all bugs and feedback as [issues in this Github repo](https://github.com/solana-labs/tour-de-sol/issues). 

Given the fast pace of communication in the Discord channels, it’s likely issues reported in them may be lost in the information flow. Filing the issues in the Github repo is the only way to ensure the issues get logged and addressed.

## 1.10 Resources

* * Current validator documentation
  * Validator chat channels
    * [\#validator-support](https://discord.gg/rZsenD) General support channel for any Validator related queries that don’t fall under Tour de SOL. 
    * [\#tourdesol](https://discord.gg/BdujK2) Discussion and support channel for Tour de SOL participants. 
    * [\#tourdesol-announcements](https://discord.gg/Q5TxEC) The single source of truth for critical information relating to Tour de SOL.
    * [\#tourdesol-stage0 D](https://discord.gg/Xf8tES)iscussion for events within Tour de SOL Stage 0. Stage 0 includes all the dry-runs.
  * [Core software repo](https://github.com/solana-labs/solana)
  * [Current Testnet/TdS repo](https://github.com/solana-labs/tour-de-sol)
  * [Submit bugs and feedback in this repo](https://github.com/solana-labs/tour-de-sol/issues)

