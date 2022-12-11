//! Integration tests directly against the Sure oracle / prediction market
import * as anchor from '@project-serum/anchor';
import * as tribeca from '@tribecahq/tribeca-sdk';
import * as solana_contrib from '@saberhq/solana-contrib';
import * as goki from '@gokiprotocol/client';
import * as web3 from '@solana/web3.js';
import * as spl from '@solana/spl-token';
import { createMint } from '@solana/spl-token';
import { Oracle } from '../target/types/oracle';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';

const createConfigPDA = (
	tokenMint: web3.PublicKey,
	programId: web3.PublicKey
) => {
	return findProgramAddressSync(
		[Buffer.from('sure-oracle-config'), tokenMint.toBytes()],
		programId
	);
};

describe('Test Sure Prediction Market ', () => {
	const provider = anchor.AnchorProvider.env();
	const { connection } = provider;
	anchor.setProvider(provider);
	console.log('anchor.workspace.Oracle: ', anchor.workspace);
	const program = anchor.workspace.Oracle as anchor.Program<Oracle>;

	// prepare tribeca ( veSure) and goki (smart wallet) sdks
	const solanaProvider = solana_contrib.SolanaProvider.init({
		connection,
		wallet: provider.wallet,
		opts: provider.opts,
	});
	const tribecaSDK = tribeca.TribecaSDK.load({ provider: solanaProvider });
	const gokiSDK = goki.GokiSDK.load({ provider: solanaProvider });
	const minterWallet = web3.Keypair.generate();
	let sureMint: web3.PublicKey;
	before(async () => {
		const airdrop = await connection.requestAirdrop(
			minterWallet.publicKey,
			10 * web3.LAMPORTS_PER_SOL
		);
		await connection.confirmTransaction(airdrop);

		// create sure mint with 6 decimals
		sureMint = await createMint(
			connection,
			minterWallet,
			minterWallet.publicKey,
			minterWallet.publicKey,
			6
		);

		try {
			const minterWalletSureATA = await spl.createAssociatedTokenAccount(
				connection,
				minterWallet,
				sureMint,
				minterWallet.publicKey
			);

			// genesis mint of sure tokens - 500,000,000
			const genesisMintAmount = new anchor.BN(500000000).mul(
				new anchor.BN(10).pow(new anchor.BN(6))
			);
			const mintPendingTransaction = await spl.mintTo(
				connection,
				minterWallet,
				sureMint,
				minterWalletSureATA,
				minterWallet,
				BigInt(genesisMintAmount.toString())
			);
			await connection.confirmTransaction(mintPendingTransaction);
		} catch (err) {
			throw new Error(`Failed to mint Sure tokens. Cause ${err}`);
		}

		try {
			// Setup Sure governance and token locking
			const base = web3.Keypair.generate();
			const governor = await tribeca.findGovernorAddress(base.publicKey);
			const owners = [governor[0]];
			const pendingSmartWallet = await await gokiSDK.newSmartWallet({
				owners,
				threshold: new anchor.BN(1),
				numOwners: 1,
				base: base,
			});
			await pendingSmartWallet.tx.confirm();
			const smartWallet = pendingSmartWallet.smartWalletWrapper;
			const governSDK = new tribeca.GovernWrapper(tribecaSDK);
			const lockerPK = tribeca.getLockerAddress(base.publicKey);

			// create governor
			const govern = await governSDK.createGovernor({
				electorate: lockerPK,
				smartWallet: smartWallet.key,
				baseKP: base,
			});
			await govern.tx.confirm();

			// create a sure locker
			const createLocker = await tribecaSDK.createLocker({
				governor: govern.wrapper.governorKey,
				govTokenMint: sureMint,
				baseKP: base,
			});
			await createLocker.tx.confirm();
			const sureLockerPK = createLocker.locker;
		} catch (err) {
			throw new Error(`Failed to create Sure governance. Cause: ${err}`);
		}
	});

	it('Initialize config first time ->  ', async () => {
		// create a protocol authority whcih contrals the
		try {
			const protocolAuthority = web3.Keypair.generate();
			spl.createAssociatedTokenAccount(
				connection,
				minterWallet,
				sureMint,
				protocolAuthority.publicKey
			);
			const configAccount = createConfigPDA(sureMint, program.programId);
			program.methods.initializeConfig(protocolAuthority.publicKey).accounts({
				config: configAccount[0],
				tokenMint: sureMint,
			});
		} catch (err) {
			throw new Error('Failed to initialize oracle config. Cause ' + err);
		}
	});
});
