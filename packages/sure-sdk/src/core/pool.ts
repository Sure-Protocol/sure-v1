import * as anchor from '@project-serum/anchor';

import { PublicKey } from '@solana/web3.js';
import { POOL_SEED, SURE_POOLS_SEED } from './seeds';
import { Program } from '@project-serum/anchor';
import { SurePool } from './../anchor/types/sure_pool';
import { getProtocolOwner } from './protocol';

export const getPoolPDA = async (
	program: Program<SurePool>,
	smartContractToInsure: PublicKey
): Promise<anchor.web3.PublicKey> => {
	const [poolPDA, poolBump] = await PublicKey.findProgramAddress(
		[POOL_SEED, smartContractToInsure.toBytes()],
		program.programId
	);
	return poolPDA;
};

export const getSurePools = async (
	program: Program<SurePool>
): Promise<PublicKey> => {
	const [surePoolsPDA, surePoolsBump] = await PublicKey.findProgramAddress(
		[SURE_POOLS_SEED],
		program.programId
	);

	return surePoolsPDA;
};

export const createPool = async (
	program: Program<SurePool>,
	wallet: anchor.Wallet,
	smartContractAddress: anchor.web3.PublicKey,
	insuranceFee: number
) => {
	const [protocolOwnerPDA, protocolOwnerBump] = await getProtocolOwner(
		program.programId
	);
	const poolPDA = await getPoolPDA(program, smartContractAddress);
	const surePoolsPDA = await getSurePools(program);

	try {
		await program.methods
			.createPool(insuranceFee, 'name')
			.accounts({
				poolCreator: wallet.publicKey,
				protocolOwner: protocolOwnerPDA,
				pool: poolPDA,
				surePools: surePoolsPDA,
				insuredTokenAccount: smartContractAddress,
				rent: anchor.web3.SYSVAR_RENT_PUBKEY,
				systemProgram: program.programId,
			})
			.rpc();
	} catch (err) {
		throw new Error('Could not create pool. Cause: ' + err);
	}
};
