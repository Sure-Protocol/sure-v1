import * as web3 from '@solana/web3.js';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import type { SureOracleSDK } from '@surec/oracle';

export * from './time';
export * from './sdks';
export * from './money';
export * from './formatting';
export function getTestKeypairFromSeed(oracleSdk: SureOracleSDK, seed: string): web3.Keypair {
	console.log('oracleSdk.program.programId: ', oracleSdk.program.programId);
	const [pda] = findProgramAddressSync([Buffer.from(seed)], oracleSdk.program.programId);
	return web3.Keypair.fromSeed(pda.toBytes());
}
