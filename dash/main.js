import {
  Connection,
  PublicKey,
  VoteAccount,
  VOTE_ACCOUNT_KEY,
} from '@solana/web3.js';

export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

const connection = new Connection('http://tds.solana.com:8899');
const nodeConnectionCache = {};

async function dashboard() {
  console.log(`--- ${new Date()} ---`);

  const leader = await connection.getSlotLeader();
  const clusterNodes = await connection.getClusterNodes();
  const epochVoteAccounts = await connection.getEpochVoteAccounts();
  const allVoteAccounts = await connection.getProgramAccounts(VOTE_ACCOUNT_KEY);

  const nodes = {};
  for (const epochVoteAccount of epochVoteAccounts) {
    const {nodePubkey, stake, votePubkey} = epochVoteAccount;
    nodes[nodePubkey] = {
      stake,
      votePubkey,
    };
  }

  for (const clusterNode of clusterNodes) {
    const {pubkey, rpc, tpu} = clusterNode;
    nodes[pubkey] = Object.assign(nodes[pubkey] || {}, {
      online: true,
      rpc,
      tpu,
    });
  }

  for (const [votePubkey, voteAccountInfo] of allVoteAccounts) {
    const voteAccount = VoteAccount.fromAccountData(voteAccountInfo.data);

    const nodePubkey = voteAccount.nodePubkey.toString();
    const node = nodes[nodePubkey];
    if (!node) {
      continue;
    }
    if (node.votePubkey && node.votePubkey != votePubkey) {
      console.warn(`note: (${nodePubkey} has multiple vote accounts)`);
      continue;
    }
    node.voteAccount = voteAccount;
    node.votePubkey = votePubkey;
  }

  const SEP = "  ";

  let log = "Role".padEnd(9);
  log += SEP + "Account".padEnd(44);
  log += SEP + "Cur. Slot".padEnd(9);
  log += SEP + "Vote Account".padEnd(44);
  log += SEP + "Root Slot".padEnd(9);
  log += SEP + "Balance".padEnd(14);
  log += SEP + "Stake".padEnd(14);
  log += SEP + "RPC Endpoint".padEnd(18)
  console.log(log);

  for (const node of Object.keys(nodes).sort()) {
    const {stake, votePubkey, voteAccount, online, rpc, tpu} = nodes[node];

    const lamports = await connection.getBalance(new PublicKey(node));
    let currentSlot = null;
    if (rpc) {
      try {
        let nodeConnection = nodeConnectionCache[rpc];
        if (nodeConnection === undefined) {
          nodeConnectionCache[rpc] = nodeConnection = new Connection(`http://${rpc}`);
        }
        currentSlot = await nodeConnection.getSlot();
      } catch (err) {
        currentSlot = 'error';
      }
    }

    let what;
    if (node === leader) {
      what = 'Leader';
    } else if (!tpu && online) {
      what = 'Spy';
    } else {
      what = 'Validator';
    }
    if (!online) {
      what = `OFFLINE! ${what}`;
    }
    let log = `${what}`.padStart(9);
    log += SEP + `${node}`.padStart(44);
    log += SEP + (currentSlot !== null ? `${currentSlot}` : '').padStart(9);
    log += SEP + (voteAccount ? `${votePubkey}` : 'None').padStart(44);
    log += SEP + (voteAccount ? `${voteAccount.rootSlot}` : 'N/A').padStart(9);
    log += SEP + `${lamports}`.padStart(14);
    log += SEP + (stake ? `${stake}` : 'None').padStart(14);
    log += SEP + (rpc ? `http://${rpc}` : '').padStart(18);
    console.log(log);
  }
}

async function main() {
  for (;;) {
    try {
      await dashboard();
    } catch (err) {
      console.log(err);
    }
    await sleep(5000);
  }
}

main().catch(err => {
  console.log(err);
  process.exit(1);
});
