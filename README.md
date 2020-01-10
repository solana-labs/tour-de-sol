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
    --exclude-pubkey rpc1io1gmhuEq26wTBARGJfGGw48S7GYaHfKVEf9Dvv \
    --exclude-pubkey va11wrZ2pD668e2dKXohuXiyALPxfVQjjH7zzpePavQ \
    --exclude-pubkey va12u4o9DipLEB2z4fuoHszroq1U9NcAB9aooFDPJSf \
    --exclude-pubkey va13en4eUarJtf8mbhFF386nvQh12g6ESkjoR7Ji8hm \
    --exclude-pubkey 5n8KCdzqtvTnhkvCrFR7errH6ZUp11kL97r2awXkfzFe \
    --exclude-pubkey 7suRNpX7bJsXphHJtBv4ZsLjJZ1dTGeX256pLqJZdEAm \
    --exclude-pubkey 2te46rxywMdCNdkvjumiBBPQoVczJFxhxEaxFavQNqe3 \
    --exclude-pubkey ChorusXqjLC2NbiStKR6k9WoD7wu6TVTtFG8qCL5XBVa \
    --exclude-pubkey GeZ5PrJi9muVCJiJAaFBNGoCEdxGEqTp7L2BmT2WTTy1 \
    --exclude-pubkey Fe5sLQAAT7RBT8mcH1AAGCbExJQcYxcwXvp1GjrGbvxs \
    --exclude-pubkey 44e8VyWoyZSE2oYHxMHMedAiHkGJqJgPd3tdt6iKoAFL \
    --exclude-pubkey Ez4iUU87ViJLCnmSy1t1Ti3DLoysFXiBseNfnRfoehyY \
    --exclude-pubkey GUdGALCHQBeqkNc2ZAht3tBXab1N5u9qJC3PAzpL54r7 \
    --exclude-pubkey HavuVVDXXsJqMzPwQ4KcF5kFm2xqjbChhyi1bgGeCQif \
    --exclude-pubkey pbAxyqHHPMwgEjv8kmjGxysk9rhNtN7q22eAjReq6Hj
```
