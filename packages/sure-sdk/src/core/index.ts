import * as anchor from '@project-serum/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import { IDL, SurePool } from './../anchor/types/sure_pool';
import {} from '@solana/web3.js';
import { Insurance } from './insurance';
import { Liquidity } from './liquidity';
import { Pool } from './pool';
import { Protocol } from './protocol';
import { TickAccount } from './tickAccount';
import { NFT } from './nft';
export * as nft from './nft';
export * as liquidity from './liquidity';
export * as insurance from './insurance';
export * as pool from './pool';
export * as protocol from './protocol';
export * as seeds from './seeds';
export * as Tick from './tickAccount';
export * as Common from './commont';
export * from './errors';

export default interface Sdk {
	readonly program: anchor.Program<SurePool>;
}

export class SureSdk {
	public readonly insurance: Insurance;
	public readonly liquidity: Liquidity;
	public readonly nft: NFT;
	public readonly pool: Pool;
	public readonly protocol: Protocol;
	public readonly tickAccount: TickAccount;
	public readonly test: boolean;

	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {
		this.insurance = new Insurance(program, connection, wallet);
		this.liquidity = new Liquidity(program, connection, wallet);
		this.nft = new NFT(program, connection, wallet);
		this.pool = new Pool(program, connection, wallet);
		this.protocol = new Protocol(program, connection, wallet);
		this.tickAccount = new TickAccount(program, connection, wallet);
	}

	static init(
		connection: anchor.web3.Connection,
		wallet: anchor.Wallet,
		programId: PublicKey
	) {
		const provider = new anchor.AnchorProvider(connection, wallet, {
			skipPreflight: false,
		});
		anchor.setProvider(provider);
		console.log('connecting to ', programId.toBase58());
		const sureProgram = new anchor.Program<SurePool>(IDL, programId, provider);

		return new this(sureProgram, connection, wallet);
	}
}
