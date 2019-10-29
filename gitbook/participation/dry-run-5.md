# Dry Run 5

Dry Run 5 will test the performance of a heterogeneous cluster of validators under heavier transaction load. It will be structured as a series of increasing rounds of transactions per second \(TPS\). At the end of each round the validators that survive receive additional stake for the next round. [CJR: What would cause a validator to not survive?] The rounds will continue until the cluster stops making progress.

![Ramp TPS rounds visualized](../.gitbook/assets/image%20%282%29.png)

## Cluster Parameters:

* GPUs optional [CJR: Do validators running GPUs have a better chance of surviving?]
* Solana software version: v0.20.0 [CJR: Should be updated in the FAQ]
* Target transaction rate of first round: 2,000 TPS
* Transaction rate round increment: 2,000 TPS
* Round duration: 20 minuts
* Epoch duration: 4096 slots \(approximately 27 minutes\)
* Transaction signature fee: 1 lamport
* Genesis allotment to each validator: 2 SOL \(1 SOL for initial delegation, 1 SOL for transaction fees\)

## Timeline

### Phase 1: Epoch 0 to 9: Connect and delegate  \(approx. 1 hour, 30 minutes\)

When the cluster boots, validators will have approximately 1 hour 30 minutes to connect and delegate 1 SOL of stake to themselves.

### Phase 2: Epoch 10 to 13: Stake warmup \(approx. 2 hours\)

The new validator stake, delegated in Phase 1, will take 3-4 epochs \(~2 hours\) to warmup. During this time the stake of the Solana boot nodes will be reduced as well.

### Phase 3: Epoch 14+: Ramp TPS rounds begin! \(approx. 1-2 hours per round\)

Now that all validator stake is active, the Ramp TPS program will begin running the rounds of increasing transactions-per-second until the cluster dies. Each round starts with of 20 minutes of solid transactions. After 20 minutes all validators that remain with the cluster will receive an additional stake delegation. This new stake will also take 3-4 epochs to warm up, and once warm up is complete the next round commences with an increased transaction rate.

[CJR: Should validators try and re-stake the new delegation? Can that stake be delegated right away or does it have to wait for the warm up period to complete?] 

## References

* [Ramp TPS program](https://github.com/solana-labs/tour-de-sol/tree/master/ramp-tps)
