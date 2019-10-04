# Validator catch-up

Once your validator is connected and voting, it should appear on the [Solana Network Explorer](http://explorer.solana.com/validators). \(Switch to the Tour de SOL network in the top right\)

But your validator may take some time to catch up with the cluster. Use the `get-slot` wallet command to view the current slot that the network is processing: 

```text
$ solana get-slot
```

The current slot that your validator is processing can be seen with:

```text
$ solana --url http://127.0.0.1:8899 get-slot
```

To see both values at once, run:

```text
$ echo "me: $(solana --url http://127.0.0.1:8899 get-slot | grep '^[0-9]\+$'), cluster: $(solana --url http://tds.solana.com:8899 get-slot | grep '^[0-9]\+$')"
```

Your validator is caught-up when your validator’s current slot matches the current slot the network is processing.

Until your validator has caught up, it will not be able to vote successfully and stake cannot be delegated to it.

Also if you find the network's slot advancing faster than yours, you will likely never catch up. This typically implies some kind of networking issue between your validator and the rest of the network.

