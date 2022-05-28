import * as anchor from '@project-serum/anchor';

import { PublicKey } from '@solana/web3.js';
import { SURE_TICK_SEED } from './seeds';
import { SurePool } from '../anchor/types/sure_pool';
import { Program } from '@project-serum/anchor';

export const getTickAccountPDA = async (
	program: Program<SurePool>,
	poolPDA: PublicKey,
	tokenMint: PublicKey,
	tick: number
): Promise<PublicKey> => {
	let tickBN = new anchor.BN(tick);
	const [tickAccountPDA, tickAccountBump] = await PublicKey.findProgramAddress(
		[
			SURE_TICK_SEED,
			poolPDA.toBytes(),
			tokenMint.toBytes(),
			tickBN.toBuffer('le', 2),
		],
		program.programId
	);
	return tickAccountPDA;
};

/**
 * Current tick position in tick pool
 *
 * @param poolPDA PDA for pool
 * @param tick Tick in basis points to supply liquidity to
 * @param tokenMint The mint of the token to be supplied to the pool. This could be USDC
 * @return Nothing
 */
export const getCurrentTickPosition = async (
	program: Program<SurePool>,
	poolPDA: PublicKey,
	tokenMint: PublicKey,
	tick: number
): Promise<number> => {
	const tickPDA = await getTickAccountPDA(program, poolPDA, tokenMint, tick);
	try {
		const tickAccount = await program.account.tick.fetch(tickPDA);
		return tickAccount.lastLiquidityPositionIdx;
	} catch (e) {
		throw new Error('Tick account does not exist. Cause: ' + e);
	}
};

export const getOrCreateTickAccount = async (
	program: Program<SurePool>,
	poolPDA: PublicKey,
	tokenMint: PublicKey,
	tick: number,
	owner: PublicKey
): Promise<anchor.web3.PublicKey> => {
	const tickAccountPDA = await getTickAccountPDA(
		program,
		poolPDA,
		tokenMint,
		tick
	);

	try {
		await program.account.tick.fetch(tickAccountPDA);
	} catch (e) {
		console.log(
			'sure.getTickAccount.error Could not fetch tick account. Cause: ' + e
		);
		// create account
		try {
			await createTickAccount(program, poolPDA, tokenMint, tick, owner);
		} catch (e) {
			throw new Error(
				'sure.createTickAccount.error. could not create tick account. cause: ' +
					e
			);
		}
	}
	return tickAccountPDA;
};

/// Check if tick account exists for the pool,
/// if not, create the account.
export const createTickAccount = async (
	program: Program<SurePool>,
	poolPDA: PublicKey,
	tokenMint: PublicKey,
	tick: number,
	creator: PublicKey
): Promise<PublicKey> => {
	const tickAccountPDA = await getTickAccountPDA(
		program,
		poolPDA,
		tokenMint,
		tick
	);

	try {
		await program.rpc.initializeTick(poolPDA, tokenMint, tick, {
			accounts: {
				creator: creator,
				tickAccount: tickAccountPDA,
				systemProgram: program.programId,
			},
		});
	} catch (e) {
		console.log('logs?: ', e.logs);
		throw new Error('Could not create tick account: ' + e);
	}

	return tickAccountPDA;
};
