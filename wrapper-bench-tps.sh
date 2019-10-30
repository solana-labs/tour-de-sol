#!/usr/bin/env bash
set -e
exec > wrapper-bench-tps.log
exec 2>&1

tdsDir="$(pwd)"
netDir=$1
clientId=$2
txCount=$3
threadBatchSleepMs=$4

eval "$("$netDir/gce.sh" info --eval)"
clientIp="NET_CLIENT${clientId}_IP"
validatorIp="NET_VALIDATOR0_IP"

"$netDir/scp.sh" $tdsDir/bench-tps.sh solana@${!clientIp}:solana/
"$netDir/ssh.sh" ${!clientIp} solana/bench-tps.sh ${!validatorIp} $txCount $threadBatchSleepMs remote
