export * from './proposal.js';
export * from './vote.js';
export * from './sdk.js';
export * from './config.js';
export * from './constants.js';
export * from './utils.js';
export * from './program.js';
export * from './config.js';

import {
	TransactionReceipt,
	SolanaProvider,
	SolanaAugmentedProvider,
} from '@saberhq/solana-contrib';
export class TransactionResult extends TransactionReceipt {}
export class SureProvider extends SolanaProvider {}
export class SureAugmentedProvider extends SolanaAugmentedProvider {}
