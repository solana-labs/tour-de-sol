# Tour de SOL

## Validator Public Key Registration
In order to obtain your allotement of lamports at the start of a Tour de SOL stage, you need to publish your validator's identity public key under your keybase.io account.

**If this registration is not completed by the cut-off time you will not be able to participate.**

1. If you haven't already, generate your validator's identity keypair by running:
     ```bash
     $ solana-keygen new -o ~/validator-keypair.json
     Wrote /Users/<your user name>/validator-keypair.json
     ```
2. The identity public key can now obtained by running:
     ```bash
     $ solana-keygen pubkey ~/validator-keypair.json 
     <Base58 Public Key>
     ```
3. Install [Keybase](https://keybase.io/download) on your machine.
3. Create a Solana directory in your public file folder: `mkdir /keybase/public/<keybase id>/solana`
4. Publish your validator's identity public key by creating an empty file in your Keybase public file folder in the following format: `/keybase/public/<keybase id>/solana/validator-<Base58 Public Key>`.   For example:
     ```bash
     $ mkdir -p /keybase/public/<keybase id>/solana
     $ /keybase/public/<keybase id>/solana/validator-<Base58 Public Key>
     ```
