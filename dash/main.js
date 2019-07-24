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

  for (const node in nodes) {
    const {stake, votePubkey, voteAccount, online, rpc, tpu} = nodes[node];
    const lamports = await connection.getBalance(new PublicKey(node));

    let what;
    if (node === leader) {
      what = 'Leader';
    } else if (!tpu && online) {
      what = 'Spy';
    } else if (!votePubkey) {
      what = 'Blockstreamer';
    } else {
      what = 'Validator';
    }
    if (!online) {
      what = `OFFLINE! ${what}`;
    }
    let log = `${what}`.padEnd(19);
    log += `${node.padEnd(44)} | `;
    log += (voteAccount ? `root slot=${voteAccount.rootSlot}` : '').padEnd(17);
    log += `balance=${lamports}`.padEnd(20);
    log += (stake ? `stake=${stake}` : '').padEnd(18);
    if (rpc) {
      log += `rpc=http://${rpc}`;
    }

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
