import * as web3 from '@solana/web3.js';
import * as anchor from '@project-serum/anchor';
export const SURE_ADDRESSES = {
	Oracle: new web3.PublicKey('Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS'),
};

/// SEEDS
export const SURE_ORACLE_SEED = anchor.utils.bytes.utf8.encode('sure-oracle');
