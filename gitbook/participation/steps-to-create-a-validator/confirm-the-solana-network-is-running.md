# Confirm the Solana network is running

Before you connect your validator to the Solana network, confirm the network is running. To do this, view the existing nodes in the network using:

```text
$ solana-gossip --entrypoint tds.solana.com:8001 spy
```

If you see more than 1 node listed in the output of the above command, the network is running.

The `ping` command can be used to check that the cluster is able to process transactions:

```text
$ solana ping
```

This command sends a tiny transaction every 2 seconds and reports how long it takes to confirm it.

