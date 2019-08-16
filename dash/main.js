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
  const ROLE_PAD = 9;
  const ACCOUNT_PAD = 44;
  const CUR_SLOT_PAD = 9;
  const VOTE_ACCOUNT_PAD = 44;
  const ROOT_SLOT_PAD = 9;
  const BALANCE_PAD = 14;
  const STAKE_PAD = 14;
  const RPC_PAD = 20;

  let log = "Role".padEnd(ROLE_PAD);
  log += SEP + "Account".padEnd(ACCOUNT_PAD);
  log += SEP + "Cur. Slot".padEnd(CUR_SLOT_PAD);
  log += SEP + "Vote Account".padEnd(VOTE_ACCOUNT_PAD);
  log += SEP + "Root Slot".padEnd(ROOT_SLOT_PAD);
  log += SEP + "Balance".padEnd(BALANCE_PAD);
  log += SEP + "Stake".padEnd(STAKE_PAD);
  log += SEP + "RPC Endpoint".padEnd(RPC_PAD)
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
    let log = `${what}`.padStart(ROLE_PAD);
    log += SEP + `${node}`.padStart(ACCOUNT_PAD);
    log += SEP + (currentSlot !== null ? `${currentSlot}` : '').padStart(CUR_SLOT_PAD);
    log += SEP + (voteAccount ? `${votePubkey}` : 'None').padStart(VOTE_ACCOUNT_PAD);
    log += SEP + (voteAccount ? `${voteAccount.rootSlot}` : 'N/A').padStart(ROOT_SLOT_PAD);
    log += SEP + `${lamports}`.padStart(BALANCE_PAD);
    log += SEP + (stake ? `${stake}` : 'None').padStart(STAKE_PAD);
    log += SEP + (rpc ? `http://${rpc}` : '').padStart(RPC_PAD);
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
