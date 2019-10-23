#!/usr/bin/env bash
set -e
exec > wrapper-bench-tps.log
exec 2>&1

tdsDir="$(pwd)"
here=$(dirname "$0")
netDir=$1
clientId=$2
txCount=$3

cd $netDir
eval "$(./gce.sh info --eval)"
clientIp="NET_CLIENT${clientId}_IP"

./scp.sh $tdsDir/remote-bench-tps.sh solana@${!clientIp}:.
./ssh.sh ${!clientIp} killall solana-bench-tps || true
./ssh.sh ${!clientIp} ./remote-bench-tps.sh $NET_VALIDATOR0_IP $txCount

cd $here
