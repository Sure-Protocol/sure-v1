export * from './proposal';
export * from './vote';
export * from './sdk';
export * from './config';

export * from './constants';
export * from './utils';
export * from './program';
export * from './config';

import {
	TransactionReceipt,
	SolanaProvider,
	SolanaAugmentedProvider,
} from '@saberhq/solana-contrib';
export class TransactionResult extends TransactionReceipt {}
export class SureProvider extends SolanaProvider {}
export class SureAugmentedProvider extends SolanaAugmentedProvider {}
