# Install the Solana software

Before attempting to connect your validator to the Tour de SOL cluster, be familiar with connecting a validator to the Public Testnet as described [here](https://docs.solana.com/book/running-validator).

You can confirm the version running on the cluster entrypoint by running:

```text
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0","id":1,"method":"getVersion"}' tds.solana.com:8899
```

## Install Software

Install the Solana release [v0.20.0](https://github.com/solana-labs/solana/releases/tag/v0.20.0) on your machine by running:

```bash
curl -sSf https://raw.githubusercontent.com/solana-labs/solana/v0.19.1/install/solana-install-init.sh | sh -s - 0.20.0
```

The following output indicates a successful update:

```text
looking for latest release
downloading v0.19.1 installer
Configuration: /home/solana/.config/solana/install/config.yml
Active release directory: /home/solana/.local/share/solana/install/active_release
* Release version: 0.20.0
* Release URL: https://github.com/solana-labs/solana/releases/download/v0.20.0/solana-release-x86_64-unknown-linux-gnu.tar.bz2
Update successful
```

