import * as anchor from '@project-serum/anchor';
import {
	getAccount,
	getAssociatedTokenAddress,
	getOrCreateAssociatedTokenAccount,
} from '@solana/spl-token';

import { Account, Connection, SystemProgram } from '@solana/web3.js';
import { SureDate } from 'src/utils';
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
		const poolsPDA = await this.getSurePoolsPDA();
		try {
			await this.program.methods
				.initializeProtocol()
				.accounts({
					owner: this.wallet.publicKey,
					protocolOwner: protocolOwnerPDA,
					pools: poolsPDA,
					// program: this.program.programId,
					// programData: programData,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (err) {
			if (err.logs) {
				console.log('logs: ', err.logs);
			}
			console.log('protocolOwnerPDA: ', protocolOwnerPDA.toBase58());
			console.log('poolsPDA: ', poolsPDA.toBase58());
			throw new Error(
				'sure.sdk.protocol.initializeProtocol. programId: ' +
					this.program.programId +
					' Cause: ' +
					err
			);
		}
	}
}
