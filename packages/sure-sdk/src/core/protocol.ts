import * as anchor from '@project-serum/anchor';

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
		try {
			const [programData] = await anchor.web3.PublicKey.findProgramAddress(
				[this.program.programId.toBuffer()],
				new anchor.web3.PublicKey('BPFLoaderUpgradeab1e11111111111111111111111')
			);

			const [protocolOwnerPDA, protocolOwnerBump] =
				await this.getProtocolOwner();
			const poolsPDA = await this.getSurePools();

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
			throw new Error('sure.sdk.protocol.initializeProtocol. Cause: ' + err);
		}
	}
}
