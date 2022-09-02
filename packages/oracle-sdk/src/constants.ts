import * as web3 from '@solana/web3.js';
import * as anchor from '@project-serum/anchor';
export const SURE_ADDRESSES = {
	Oracle: new web3.PublicKey('2prR7H6LfRqwiP2iTyZG1suG4B3zU6JEpUBXWeQB66qH'),
};

/// SEEDS
export const SURE_ORACLE_SEED = anchor.utils.bytes.utf8.encode('sure-oracle');
export const SURE_ORACLE_VOTE_SEED =
	anchor.utils.bytes.utf8.encode('sure-oracle-vote');
export const SURE_ORACLE_REVEAL_ARRAY_SEED = anchor.utils.bytes.utf8.encode(
	'sure-oracle-reveal-array'
);
export const SURE_ORACLE_CONFIG_SEED =
	anchor.utils.bytes.utf8.encode('sure-oracle-config');

/// sure token

export const SURE_MINT: web3.PublicKey = new web3.PublicKey(
	'SRECjPkvN8TYEycXePc1ix3zGzZkWoYPMfJAKoJkcWj'
);

export const BASE_PK: web3.PublicKey = new web3.PublicKey(
	'Acyq4k7tJ38DyG4kppEEUF9AH1Cuiw7cGCfBuoEh8zH9'
);
