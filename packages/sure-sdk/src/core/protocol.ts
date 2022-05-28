import * as anchor from '@project-serum/anchor';

import { PublicKey, SystemProgram } from '@solana/web3.js';
import { SURE_TICK_SEED } from './seeds';
import { SurePool } from './../anchor/types/sure_pool';
import { Program } from '@project-serum/anchor';
import { getSurePools } from './pool';

export const getProtocolOwner = async (
	programId: PublicKey
): Promise<[PublicKey, number]> => {
	return await PublicKey.findProgramAddress([], programId);
};

export const initializeProtocol = async (
	program: Program<SurePool>,
	wallet: anchor.Wallet
) => {
	const [programData] = await anchor.web3.PublicKey.findProgramAddress(
		[program.programId.toBuffer()],
		new anchor.web3.PublicKey('BPFLoaderUpgradeab1e11111111111111111111111')
	);

	const protocolOwnerPDA = await getProtocolOwner(program.programId);
	const poolsPDA = await getSurePools(program);

	try {
		await program.methods.initializeProtocol().accounts({
			owner: wallet.publicKey,
			protocolOwner: protocolOwnerPDA,
			pools: poolsPDA,
			program: program.programId,
			programData: programData,
			systemProgram: SystemProgram.programId,
		});
	} catch (err) {
		throw new Error('');
	}
};
