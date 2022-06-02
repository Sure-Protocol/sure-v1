import * as anchor from '@project-serum/anchor';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Connection, PublicKey, SystemProgram } from '@solana/web3.js';
import { SurePool } from './../anchor/types/sure_pool';
import { Common } from './commont';
import { SURE_POOL_MANAGER_SEED } from './seeds';

export class Pool extends Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {
		super(program, connection, wallet);
	}

	async getPoolManager() {
		const [managerPDA, _] = await PublicKey.findProgramAddress(
			[SURE_POOL_MANAGER_SEED],
			this.program.programId
		);
		return managerPDA;
	}

	async createPool(
		tokenMint: PublicKey,
		smartContractAddress: PublicKey,
		insuranceFee: number,
		name?: string
	) {
		const [protocolOwnerPDA, protocolOwnerBump] = await this.getProtocolOwner();
		const poolPDA = await this.getPoolPDA(smartContractAddress);
		const poolsPDA = await this.getSurePoolsPDA();

		try {
			await this.program.methods
				.createPool(insuranceFee, name ? name : 'sure-untitled')
				.accounts({
					poolCreator: this.wallet.publicKey,
					protocolOwner: protocolOwnerPDA,
					pool: poolPDA,
					pools: poolsPDA,
					smartContract: smartContractAddress,
					rent: anchor.web3.SYSVAR_RENT_PUBKEY,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (err) {
			throw new Error(
				'addr: ' +
					this.program.programId +
					' sure.pool.createPool.error. Cause: ' +
					err
			);
		}
	}

	async createPoolVault(tokenMint: PublicKey, smartContractAddress: PublicKey) {
		const poolPDA = await this.getPoolPDA(smartContractAddress);
		const liquidityVaultPDA = await this.getPoolVaultPDA(poolPDA, tokenMint);
		const premiumVaultPDA = await this.getPremiumVaultPDA(poolPDA, tokenMint);
		const poolLiquidityTickBitmapPDA = await this.getPoolLiquidityTickBitmapPDA(
			poolPDA,
			tokenMint
		);

		try {
			await this.program.methods
				.createPoolVaults()
				.accounts({
					creator: this.wallet.publicKey,
					pool: poolPDA,
					poolVaultTokenMint: tokenMint,
					poolVault: liquidityVaultPDA,
					premiumVault: premiumVaultPDA,
					poolLiquidityTickBitmap: poolLiquidityTickBitmapPDA,
					rent: anchor.web3.SYSVAR_RENT_PUBKEY,
					tokenProgram: TOKEN_PROGRAM_ID,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (err) {
			throw new Error('sure.pool.createPoolVault.error. Cause: ' + err);
		}
	}

	async initializePoolManager() {
		try {
			const poolManagerPDA = await this.getPoolManager();
			await this.program.methods
				.initializePoolManager()
				.accounts({
					initialManager: this.wallet.publicKey,
					manager: poolManagerPDA,
					systemProgram: SystemProgram.programId,
				})
				.rpc();
		} catch (err) {
			throw new Error('sure.pool.initializePoolManager.error. Cause: ' + err);
		}
	}
}
