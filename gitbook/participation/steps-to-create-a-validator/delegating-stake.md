# Staking

**By default your validator will have no stake.** This means it will be ineligible to become leader.  
To delegate stake, first make sure your validator is running and has [caught up to the cluster](monitoring-your-validator.md#validator-catch-up).

More information about staking on solana can be found at [https://docs.solana.com/book/running-validator/validator-stake](https://docs.solana.com/book/running-validator/validator-stake)

## Create Stake Account

If you haven’t already done so, create a staking keypair. If you have completed this step, you should see the “validator-stake-keypair.json” in your Solana runtime directory.

```bash
solana-keygen new -o ~/validator-stake-keypair.json
```

## Delegate Stake

Now delegate 0.5 SOL to your validator by first creating your stake account:

`solana create-stake-account ~/validator-stake-keypair.json  0.5 SOL`

and then delegating that stake to your validator:

```bash
solana delegate-stake ~/validator-stake-keypair.json ~/validator-vote-keypair.json 0.5
```

{% hint style="warning" %}
Don’t delegate your remaining balance of 999.5 SOL, validator will those tokens to vote.
{% endhint %}

At the end of each slot, a validator is expected to send a vote transaction. These vote transactions are paid for by lamports from a validator's identity account.

This is a normal transaction so the standard transaction fee will apply. The transaction fee range is defined by the genesis block. The actual fee will fluctuate based on transaction load. You can determine the current fee via the [RPC API “getRecentBlockhash”](https://solana-labs.github.io/book-edge/jsonrpc-api.html#getrecentblockhash) before submitting a transaction.

Learn more about [transaction fees here](https://docs.solana.com/book/implemented-proposals/transaction-fees).

## Validator Stake Warm-up

Stakes need to warm up, and warmup increments are applied at Epoch boundaries, so it can take an hour or more for stake to come fully online.

To monitor your validator during its warmup period:

* View your vote account:`$ solana show-vote-account ~/validator-vote-keypair.json` This displays the current state of all the votes the validator has submitted to the network.
* View your stake account, the delegation preference and details of your stake:`$ solana show-stake-account ~/validator-stake-keypair.json`
* `$ solana uptime ~/validator-vote-keypair.json` will display the voting history \(aka, uptime\) of your validator over recent Epochs
* Look for log messages on your validator indicating your next leader slot: `[2019-09-27T20:16:00.319721164Z INFO solana_core::replay_stage] <VALIDATOR_IDENTITY_PUBKEY> voted and reset PoH at tick height ####. My next leader slot is ####`
* Once your stake is warmed up, you will see a stake balance listed for your validator on the [Solana Network Explorer](http://explorer.solana.com/validators)

## Monitor Your Staked Validator

Confirm your validator becomes a [leader](https://solana-labs.github.io/book/terminology.html#leader)

* After your validator is caught up, use the `$ solana balance` command to monitor the earnings as your validator is selected as leader and collects transaction fees
* Solana nodes offer a number of useful JSON-RPC methods to return information about the network and your validator's participation. Make a request by using curl \(or another http client of your choosing\), specifying the desired method in JSON-RPC-formatted data. For example:  

```bash
  // Request
  curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1, "method":"getEpochInfo"}' http://localhost:8899

  // Result
  {"jsonrpc":"2.0","result":{"epoch":3,"slotIndex":126,"slotsInEpoch":256},"id":1}
```

Helpful JSON-RPC methods:

* `getEpochInfo`[ An epoch](https://solana-labs.github.io/book/terminology.html#epoch) is the time, i.e. number of [slots](https://solana-labs.github.io/book/terminology.html?highlight=epoch#slot), for which a [leader schedule](https://solana-labs.github.io/book/terminology.html?highlight=epoch#leader-schedule) is valid. This will tell you what the current epoch is and how far into it the cluster is.
* `getVoteAccounts` This will tell you how much active stake your validator currently has. A % of the validator's stake is activated on an epoch boundary. You can learn more about staking on Solana [here](https://solana-labs.github.io/book-edge/stake-delegation-and-rewards.html).
* `getLeaderSchedule` At any given moment, the network expects only one validator to produce ledger entries. The [validator currently selected to produce ledger entries](https://solana-labs.github.io/book/leader-rotation.html?highlight=leader#leader-rotation) is called the “leader”.  This will return the complete leader schedule \(on a slot-by-slot basis\) for the current epoch. If you validator is scheduled to be leader based on its currently activated stake, the identity pubkey will show up 1 or more times here. 
* The TdS repo comes with a script to automatically make a batch of these RPC requests: [rpc-check.sh](https://github.com/solana-labs/tour-de-sol/blob/master/rpc-check.sh)

