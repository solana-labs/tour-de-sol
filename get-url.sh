url=http://tds.solana.com:8899
if [[ $1 = local ]]; then
  url=http://localhost:8899
elif [[ -n $1 ]]; then
  url=$1
fi
