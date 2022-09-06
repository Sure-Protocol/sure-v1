import * as web3 from '@solana/web3.js';
import * as anchor from '@project-serum/anchor';
import type { SureOracleSDK } from '@surec/oracle';

export * from './time';
export * from './sdks';
export * from './money';
export * from './formatting';
export * from './salt';
export function getTestKeypairFromSeed(oracleSdk: SureOracleSDK, seed: string): web3.Keypair {
	const [pda] = anchor.utils.publicKey.findProgramAddressSync(
		[Buffer.from(seed)],
		oracleSdk.program.programId
	);
	return web3.Keypair.fromSeed(pda.toBytes());
}
