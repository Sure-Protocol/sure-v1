import * as web3 from '@solana/web3.js';
import * as anchor from '@project-serum/anchor';
export const SURE_ADDRESSES = {
	Oracle: new web3.PublicKey('G3HjAD81oEXbR867NNBfpZ2PWDhsioaCguPZhTiXunu'),
};

/// SEEDS
export const SURE_ORACLE_SEED = anchor.utils.bytes.utf8.encode('sure-oracle');
export const SURE_ORACLE_VOTE_SEED =
	anchor.utils.bytes.utf8.encode('sure-oracle-vote');
export const SURE_ORACLE_REVEAL_ARRAY_SEED = anchor.utils.bytes.utf8.encode(
	'sure-oracle-reveal-array'
);

/// sure token
export const SURE_TOKEN: web3.PublicKey = new web3.PublicKey(
	'8mWJ39FzeM4ZqZsa8JSfsUf9mRdkL7AyFSrizFkFtzfi'
);
