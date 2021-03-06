import * as anchor from '@project-serum/anchor';
import {
	getAccount,
	getAssociatedTokenAddress,
	getOrCreateAssociatedTokenAccount,
} from '@solana/spl-token';

import { Connection, SystemProgram } from '@solana/web3.js';
import { SurePool } from './../anchor/types/sure_pool';
import { Common } from './commont';

export class Protocol extends Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {
		super(program, connection, wallet);
	}

	async initializeProtocol() {
		const [programData] = await anchor.web3.PublicKey.findProgramAddress(
			[this.program.programId.toBuffer()],
			new anchor.web3.PublicKey('BPFLoaderUpgradeab1e11111111111111111111111')
		);

		const [protocolOwnerPDA, protocolOwnerBump] = await this.getProtocolOwner();
		const poolsPDA = await this.getPoolsPDA();
		try {
			await this.program.methods
				.initializeProtocol()
				.accounts({
					owner: this.wallet.publicKey,
					protocolOwner: protocolOwnerPDA,
					pools: poolsPDA,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (err) {
			if (err.logs) {
				console.log('logs: ', err.logs);
			}
			throw new Error(
				'sure.sdk.protocol.initializeProtocol. programId: ' +
					this.program.programId +
					' Cause: ' +
					err
			);
		}
	}
}
