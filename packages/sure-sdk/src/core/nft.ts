import { PublicKey, AccountInfo, ParsedAccountData } from '@solana/web3.js';
import { getMint, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import * as anchor from '@project-serum/anchor';
import { TokenAccount } from '../types';
import * as protocol from './protocol';

import { Program } from '@project-serum/anchor';
import { SurePool } from './../anchor/types/sure_pool';

export const getSureNfts = async (
	connection: anchor.web3.Connection,
	wallet: PublicKey,
	program: Program<SurePool>
): Promise<Array<TokenAccount>> => {
	// Get all tokens held by wallet
	const tokensOwnedByWallet = await connection.getParsedTokenAccountsByOwner(
		wallet,
		{ programId: TOKEN_PROGRAM_ID }
	);

	const [sureMintAuthority, _] = await protocol.getProtocolOwner(
		program.programId
	);
	const sureNfts: Array<TokenAccount> = [];
	for (let t = 0; t < tokensOwnedByWallet.value.length; t++) {
		const tokenMint = new PublicKey(
			tokensOwnedByWallet.value[t].account.data.parsed.info.mint
		);
		const tokenMintAccount = await getMint(connection, tokenMint);
		if (
			tokenMintAccount?.mintAuthority?.toBase58() ===
			sureMintAuthority.toBase58()
		) {
			sureNfts.push(tokensOwnedByWallet.value[t]);
		}
	}

	return sureNfts;
};
