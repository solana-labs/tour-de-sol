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
    --exclude-pubkey pbAxyqHHPMwgEjv8kmjGxysk9rhNtN7q22eAjReq6Hj \
    --exclude-pubkey id2GQ6YwsjTCCHJ9pJaffC3MEezPscNLjPdGPSfaN46 \
    --exclude-pubkey id2QqCczwAMoDuG4sVZFjooysAqhs6hzMgGigg2rbMV \
    --exclude-pubkey id4PaxnDQwH6mLjrXZ7S56DoLxKLFdkSBmr6vma6sBf \
    --exclude-pubkey id7U3WaKEeWgGAyNydEHZFiitkc1bL5VzYmxZXTYfVY \
    --exclude-pubkey id7ywnFUjQ27BueJJc1U4inAWEvWpMaBX1fXbBKWz2J \
    --exclude-pubkey idB6NCyjMBTfdMuC9yj8vd8iiajNd9Mbk2AXp8a5xHe \
    --exclude-pubkey idBpZi4KcCoV5t88BSrnJ98zv9dZkLZQBk54TFyAdaZ \
    --exclude-pubkey idCfrfPBhvPWxj1N29n7gbMMejPraDsATyWQPFAXJuZ \
    --exclude-pubkey idEfgYfaLCvtWA7attVbBNgnfokJNpjbXpqLuTSPjUp \
    --exclude-pubkey idF6btoY9VHbnU5sCYpZ1Bu8yBAL3bKfN2k47ukan3n \
    --exclude-pubkey idGi3LekrrcVzvnQiydodwM86eqZSo8mWN59ytxZdH5 \
    --exclude-pubkey idGxeLFcK36ZQmaF249uPTZZGnRn7FpXoPJR5LKhv1v \
    --exclude-pubkey idJ8bnEkJf5CL7FngrM4zm1BHMLT1iMoQnJkL6sbq3B \
    --exclude-pubkey idKsePUfNbUALy2qiCEh1gFKgjuLc6p6pekmto9R5Cb \
    --exclude-pubkey idLdNwPPV5Zikk3sVANyXzUquXzQzwmWbib6m15WT8t \
    --exclude-pubkey idMC8bibeXspJphRNK5HpwK3Lh744fDtksrT2cULH9q \
    --exclude-pubkey idMi1g3V87WNYa3SXGLzsHeKNFaJVrryF6on5dU7DA2 \
    --exclude-pubkey idNFd41HQWr5NmPrZRwFwxXUnR4YqHpGC3CGke2KBRW \
    --exclude-pubkey idQwur4HP41cWqDxktp1UeT1PiG6KcTYKEbQeWyQ4BU \
    --exclude-pubkey idSM4aD8kSQmRxm12yvzaYES3esUeMiFmJdxaPEVYPT \
    --exclude-pubkey idSMbfe8Up3syM8sgn8Ubzd1FYc77KptdMDbufBv3GS \
    --exclude-pubkey idSPToD3qXCvFnJiQ5qRHaTQTqh2pFXmXZFZ5XkvYzV \
    --exclude-pubkey idSn2FMj47RAWVoCb2pgt8YCnfnfwY2vReQMNSPceMA \
    --exclude-pubkey idVXGKsb7F2nFRW51anXiyHPJTFVmnusnHr5zYJaABA \
    --exclude-pubkey idVcb8J66gCuAwFjLoKGuuvpxxFnhhfpENhUzR4u7tQ \
    --exclude-pubkey idVo6gQWhf1qptvnt1YwD6thehshDfzN9GZajC6nXkC \
    --exclude-pubkey idY8iDUV2VeMqeBCRCHgzNPiASXkJKQhDqWMBqH26c9 \
    --exclude-pubkey idZwgtd3r3MX7kriLRX6q3nuPqtb24ZtthS9L23SR6A \
    --exclude-pubkey idbdSGgRFPQdrsz7wtc2Tbx7MaL4xA6r98yuxrtqEyS \
    --exclude-pubkey ideNfZLPpeySDZXUnnwmzhaYTh7DX62i2C52icbaEAv \
    --exclude-pubkey idgxuQoRvB4nP9jj5HshoYWBTQk13pAisn1d96vEap4 \
    --exclude-pubkey idh5GBgrpiay2jgERL61YhfgurjvxbAG7RzPTimHahQ \
    --exclude-pubkey idjgVqqz9K7qL3eMGApzuoPsZBpUyjd3EDjBwJZPMmF \
    --exclude-pubkey idmWehGyjwQKmyw4AKG74srqW32Z8ecMMAzJM4pFami \
    --exclude-pubkey idrpuShZXt12i7tGrN2gfYQPD7Xvak4Ai23WkqYeSPt \
    --exclude-pubkey idsdFzMbYy9D2YmFm9QmFMi73FsKN4nV7WV8zMUcMEq \
    --exclude-pubkey iduRkR7DKVWz34HEPAQXVUhh1tVC9wdwTNXcif4CVdq \
    --exclude-pubkey iduwx2nrXM6WmHSF7AtxdV4LbZgEdXN866xYdZDx9wB \
    --exclude-pubkey idxVaEn8v3zGTxqJbevTzjJhHdkA5p68ahcXfBqrka8 \
    --exclude-pubkey idz77S9k24pczEcgB4edV9uQag8ZuzZ3NK3DjxroaLq \
    --exclude-pubkey idzabijKtknbbsmXVB65wrb4PaMdgGjpHhGJ9psD4d2 \
    --exclude-pubkey ide8fez9zNJBJwaESdAo5xtuk9FuWQRVjUjcaXWnm3D
```
