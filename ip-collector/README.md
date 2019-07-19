Collects all IP addresses of validators participating in the TdS cluster by
polling every N minutes.

Usage:
```bash
$ yarn
$ yarn run main
```

All observed validator IP addresses will be written into the file
`observed-ip-addresses.json` indexed by their public key.

This file will persist across restarts of the program.
