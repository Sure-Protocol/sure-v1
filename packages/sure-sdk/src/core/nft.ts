import { PublicKey, Connection } from '@solana/web3.js';
import { getMint, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import * as anchor from '@project-serum/anchor';
import { TokenAccount } from '../types';

import { SurePool } from './../anchor/types/sure_pool';
import { SureSdk } from '.';
import { Common } from './commont';

export class NFT extends Common {
	constructor(
		readonly program: anchor.Program<SurePool>,
		readonly connection: Connection,
		readonly wallet: anchor.Wallet
	) {
		super(program, connection, wallet);
	}

	getSureNfts = async (): Promise<Array<TokenAccount>> => {
		// Get all tokens held by wallet
		const tokensOwnedByWallet =
			await this.connection.getParsedTokenAccountsByOwner(
				this.wallet.publicKey,
				{
					programId: TOKEN_PROGRAM_ID,
				}
			);

		const [sureMintAuthority, _] = await this.getProtocolOwner();
		const sureNfts: Array<TokenAccount> = [];
		for (let t = 0; t < tokensOwnedByWallet.value.length; t++) {
			const tokenMint = new PublicKey(
				tokensOwnedByWallet.value[t].account.data.parsed.info.mint
			);
			const tokenMintAccount = await getMint(this.connection, tokenMint);
			if (
				tokenMintAccount?.mintAuthority?.toBase58() ===
				sureMintAuthority.toBase58()
			) {
				sureNfts.push(tokensOwnedByWallet.value[t]);
			}
		}

		return sureNfts;
	};
}
