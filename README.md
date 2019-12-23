## Solana's Tour de SOL
Please see https://docs.solana.com/tour-de-sol/ for information on how to
participate in Tour de SOL.

### Winner Tool
For transparency, we have included the tool we will be using for calculating the
winners of the quantitative reward categories. You can find more details on how
winners are calculated by reading the [forum announcement].

[forum announcement]: https://forums.solana.com/t/tour-de-sol-stage-1-preliminary-compensation-design/79

```bash
$ solana-tds-winner-tool --ledger /path/to/tds/ledger \
  --baseline-validator boot1Z6jb15CLqpaMTn2CxktktwZpRAVAgHZEW6SxQ7 \
  --exclude-pubkeys rpc1io1gmhuEq26wTBARGJfGGw48S7GYaHfKVEf9Dvv \
  --exclude-pubkeys va11wrZ2pD668e2dKXohuXiyALPxfVQjjH7zzpePavQ \
  --exclude-pubkeys va12u4o9DipLEB2z4fuoHszroq1U9NcAB9aooFDPJSf \
  --exclude-pubkeys va13en4eUarJtf8mbhFF386nvQh12g6ESkjoR7Ji8hm
```
