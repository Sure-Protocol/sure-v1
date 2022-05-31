import * as anchor from '@project-serum/anchor';

export const POOL_SEED = anchor.utils.bytes.utf8.encode('sure-insurance-pool');
export const TOKEN_VAULT_SEED = anchor.utils.bytes.utf8.encode('sure-ata');
export const SURE_BITMAP = anchor.utils.bytes.utf8.encode('sure-bitmap');
export const SURE_LIQUIDITY_POSITION =
	anchor.utils.bytes.utf8.encode('sure-lp');
export const SURE_TICK_SEED = anchor.utils.bytes.utf8.encode('sure-tick');
export const SURE_VAULT_POOL_SEED = anchor.utils.bytes.utf8.encode(
	'sure-liquidity-vault'
);
export const SURE_PREMIUM_POOL_SEED =
	anchor.utils.bytes.utf8.encode('sure-premium-vault');
export const SURE_NFT_MINT_SEED = anchor.utils.bytes.utf8.encode('sure-nft');
export const SURE_TOKEN_ACCOUNT_SEED =
	anchor.utils.bytes.utf8.encode('sure-token-account');
export const SURE_MP_METADATA_SEED = anchor.utils.bytes.utf8.encode('metadata');
export const SURE_INSURANCE_CONTRACT = anchor.utils.bytes.utf8.encode(
	'sure-insurance-contract'
);
export const SURE_INSURANCE_CONTRACTS = anchor.utils.bytes.utf8.encode(
	'sure-insurance-contracts'
);
export const SURE_POOLS_SEED = anchor.utils.bytes.utf8.encode('sure-pools');

export const SURE_POOL_MANAGER_SEED =
	anchor.utils.bytes.utf8.encode('sure-pool-manager');
