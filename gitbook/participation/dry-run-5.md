# Dry Run 5

Dry Run 5 will test how well a heterogeneous cluster of validators perform under heavier transaction load.  It will be structured as a series of increasing rounds of transactions per second \(TPS\). At the end of each round the validators that survive receive additional stake for the next round.  The rounds will continue until the cluster fails to maintain consensus.

## Cluster Parameters:

* GPUs are not required but may be used
* Epoch duration: 4096 slots \(approximately 27 minutes at 400ms per slot\)
* Signature fee: 1 lamport
* Solana software version: v0.20.0
* Target transaction rate of first round: 2,000 TPS
* Transaction rate increment between rounds: 2000 TPS

## Timeline

### Epoch 0 to 9: Connect and Delegate  \(approx. 1 hour, 30 minutes\)

When the cluster boots, validators will have approximately 1 hour 30 minutes to connect and delegate 1 SOL of stake to themselves.

### Epoch 10 to 13: Stake warmup \(approx. 2 hours\)

With all online and validators delegated to, the new stake will take 3-4 epochs \(~2 hours\) to warmup.

### Epoch 14+: Ramp TPS rounds begin! \(approx. 1-2 hours per round\)

Now that all validators are staked, the Ramp TPS program will begin running rounds of increasing transactions-per-second until the cluster dies.  Each round starts with of 20 minutes of solid transactions.  After 20 minutes, all validators that remain with the cluster will receive an additional delegation of stake.  This new stake will also take 3-4 epochs to warm up.  Once warm up is complete, the next round commences with an increased transaction rate.

![Ramp TPS rounds visualized](../.gitbook/assets/image.png)



