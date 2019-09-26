# Publishing Information About Your Validator

See [https://docs.solana.com/book/running-validator/validator-info](https://docs.solana.com/book/running-validator/validator-info) for background:

Example publish command:

```bash
solana validator-info publish ~/validator-keypair.json "Elvis Validator" -n elvis -w "https://elvis-validates.com"
```

Example query command:

```bash
solana validator-info get
```
which outputs
```
Validator info from 8WdJvDz6obhADdxpGCiJKZsDYwTLNEDFizayqziDc9ah
  Validator pubkey: 6dMH3u76qZ7XG4bVboVRnBHR2FfrxEqTTTyj4xmyDMWo
  Info: {"keybaseUsername":"elvis","name":"Elvis Validator","website":"https://elvis-validates.com"}
```



