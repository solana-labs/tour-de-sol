#!/usr/bin/env bash
set -e
exec > wrapper-bench-tps.log
exec 2>&1

tdsDir="$(pwd)"
netDir=$1
clientId=$2
txCount=$3

eval "$("$netDir/gce.sh" info --eval)"
clientIp="NET_CLIENT${clientId}_IP"

"$netDir/scp.sh" $tdsDir/remote-bench-tps.sh solana@${!clientIp}:.
"$netDir/ssh.sh" ${!clientIp} killall solana-bench-tps || true
"$netDir/ssh.sh" ${!clientIp} ./remote-bench-tps.sh $NET_VALIDATOR0_IP $txCount
