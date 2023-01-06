import { AnchorError } from '@project-serum/anchor';
import * as web3 from '@solana/web3.js';
import { topUpAccount, topUpSure, topUpVeSure } from '.';
import * as anchor from '@project-serum/anchor';
import * as tribeca from '@tribecahq/tribeca-sdk';

/**
 *
 * @param param0
 * @returns
 */
export const generateTestVoter = async <T extends anchor.Idl>({
	program,
	mint,
	minterWallet,
	tribecaSDK,
	sureLocker,
	governor,
}: {
	mint: web3.PublicKey;
	minterWallet: web3.Signer;
	program: anchor.Program<T>;
	tribecaSDK: tribeca.TribecaSDK;
	sureLocker: web3.PublicKey;
	governor: web3.PublicKey;
}): Promise<[web3.PublicKey, web3.Keypair]> => {
	const voter = web3.Keypair.generate();
	await topUpAccount({
		connection: program.provider.connection,
		pk: voter.publicKey,
	});
	await topUpSure({
		connection: program.provider.connection,
		mint,
		minterWallet,
		to: voter.publicKey,
		amount: 200,
	});

	// lockup some some
	await topUpVeSure({
		program,
		tribecaSDK,
		sureLocker,
		governor,
		mint,
		voter: voter,
		amount: 100,
	});

	const lockerWrapper = await tribeca.LockerWrapper.load(
		tribecaSDK,
		sureLocker,
		governor
	);
	const escrowRes = await lockerWrapper.getOrCreateEscrow(voter.publicKey);
	return [escrowRes.escrow, voter];
};
