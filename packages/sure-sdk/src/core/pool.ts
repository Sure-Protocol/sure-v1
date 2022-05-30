import * as anchor from '@project-serum/anchor';
import { Connection } from '@solana/web3.js';
import { SurePool } from './../anchor/types/sure_pool';
import { SureSdk } from '.';
import { Common } from './commont';

export class Pool extends Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {
		super(program, connection, wallet);
	}

	async createPool(
		smartContractAddress: anchor.web3.PublicKey,
		insuranceFee: number
	) {
		const [protocolOwnerPDA, protocolOwnerBump] = await this.getProtocolOwner();
		const poolPDA = await this.getPoolPDA(smartContractAddress);
		const surePoolsPDA = await this.getSurePools();

		try {
			await this.program.methods
				.createPool(insuranceFee, 'name')
				.accounts({
					poolCreator: this.wallet.publicKey,
					protocolOwner: protocolOwnerPDA,
					pool: poolPDA,
					surePools: surePoolsPDA,
					insuredTokenAccount: smartContractAddress,
					rent: anchor.web3.SYSVAR_RENT_PUBKEY,
					systemProgram: this.program.programId,
				})
				.rpc();
		} catch (err) {
			throw new Error('Could not create pool. Cause: ' + err);
		}
	}
}
