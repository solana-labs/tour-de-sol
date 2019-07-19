const solanaWeb3 = require('@solana/web3.js');
const fs = require('fs');

const connection = new solanaWeb3.Connection('http://tds.solana.com:8899');

function logError(err) {
  console.log(err.message);
}

let observed = {};
try {
  observed = JSON.parse(fs.readFileSync('observed-ip-addresses.json'));
  console.log('Loaded observed-ip-addresses.json');
} catch (err) {
  console.log(err.message);
}

function collectIps() {
  console.log('Fetching cluster nodes...');
  connection.getClusterNodes()
  .then(nodes => {
    let write = false;
    for (const node of nodes) {
      const ip = node.gossip.split(':')[0]
      const {pubkey} = node;
      if (!observed[pubkey]) {
        observed[pubkey] = [ip];
        write = true;
      } else {
        if (!observed[pubkey].some(i => i === ip)) {
          observed[pubkey].push(ip);
          write = true;
        }
      }
    }

    if (write) {
      const data = JSON.stringify(observed);
      console.log(data);
      fs.writeFileSync('observed-ip-addresses.json', data);
    } else {
      console.log('No change');
    }
  })
  .catch(logError)
  .then(() => setTimeout(collectIps, 1000 * 60 * 10)); // Poll every 10 minutes
}

collectIps();
