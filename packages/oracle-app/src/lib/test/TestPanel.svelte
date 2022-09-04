<script lang="ts">
	import { css } from '@emotion/css';
	import { globalStore, newEvent } from '$stores/index';
	import * as web3 from '@solana/web3.js';
	import * as spl from './../../../node_modules/@solana/spl-token';
	import {
		createAccount,
		createAssociatedTokenAccountInstruction,
		createInitializeMintInstruction,
		createMint,
		createMintToInstruction,
		getAccount,
		getAssociatedTokenAddress,
		getMinimumBalanceForRentExemptMint,
		getOrCreateAssociatedTokenAccount,
		mintTo,
		MINT_SIZE
	} from './../../../node_modules/@solana/spl-token';
	import * as tribeca from '@tribecahq/tribeca-sdk';
	import * as goki from '@gokiprotocol/client';
	import * as anchor from '@project-serum/anchor';
	import type NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
	import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
	import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
	import { createInitMintInstructions, getTokenAccount } from '@saberhq/token-utils';
	import { walletStore } from '@svelte-on-solana/wallet-adapter-core';
	import { ASSOCIATED_PROGRAM_ID } from '@project-serum/anchor/dist/cjs/utils/token';
	import { getTestKeypairFromSeed } from '$lib/utils';
	import { SURE_MINT } from '@surec/oracle';

	$: tribecaSdk = $globalStore?.oracleSDK?.provider
		? tribeca.TribecaSDK.load({ provider: $globalStore.oracleSDK.provider })
		: undefined;

	$: gokiSdk = $globalStore?.oracleSDK?.provider
		? goki.GokiSDK.load({ provider: $globalStore.oracleSDK?.provider })
		: undefined;

	async function createCustomMint() {
		const oracleSdk = $globalStore.oracleSDK;

		if (oracleSdk && $globalStore.wallet?.publicKey) {
			const mintKeypair = getTestKeypairFromSeed(oracleSdk, 'sure_test_8');
			try {
				spl.createMint(
					oracleSdk.provider.connection,
					(oracleSdk.provider.wallet as NodeWallet).payer,
					oracleSdk.provider.walletKey,
					null,
					6
				);
				const tx = new web3.Transaction().add(
					web3.SystemProgram.createAccount({
						fromPubkey: oracleSdk.provider.wallet.publicKey,
						newAccountPubkey: mintKeypair.publicKey,
						space: MINT_SIZE,
						lamports: await getMinimumBalanceForRentExemptMint(oracleSdk.provider.connection),
						programId: TOKEN_PROGRAM_ID
					}),
					createInitializeMintInstruction(
						mintKeypair.publicKey,
						6,
						oracleSdk.provider.wallet.publicKey,
						null
					)
				);

				const txRes = await oracleSdk.provider.send(tx, [mintKeypair]);
				const txRec = await txRes.confirm({});
				newEvent.set({
					name: `created mint: ${mintKeypair.publicKey}`,
					status: 'success',
					tx: txRec
				});
			} catch (err) {
				const error = err as web3.SendTransactionError;
				newEvent.set({ name: `could not create mint account`, status: 'error', tx: error.message });
			}
		}
	}

	async function mintToUser() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && $globalStore.wallet?.publicKey && $globalStore.provider) {
			try {
				const mintPk = SURE_MINT;
				const ataPDA = await getAssociatedTokenAddress(mintPk, oracleSdk.provider.wallet.publicKey);
				const tx = new web3.Transaction();
				try {
					const account = await getAccount(oracleSdk.provider.connection, ataPDA);
				} catch {
					tx.add(
						createAssociatedTokenAccountInstruction(
							oracleSdk.provider.wallet.publicKey,
							ataPDA,
							oracleSdk.provider.wallet.publicKey,
							mintPk
						)
					);
				}

				tx.add(
					createMintToInstruction(mintPk, ataPDA, oracleSdk.provider.wallet.publicKey, 100_000_000)
				);

				const txRes = await oracleSdk.provider.send(tx);
				const txRec = await txRes.confirm({});
				newEvent.set({ name: `minted 100 sure tokens`, status: 'success', tx: txRec });
			} catch (err) {
				const error = err as web3.SendTransactionError;
				newEvent.set({ name: `could not mint new tokens`, status: 'error', tx: error.message });
			}
		}
	}
	async function createSmartWallet() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && gokiSdk && tribecaSdk) {
			try {
				const base = getTestKeypairFromSeed(oracleSdk, 'sure_base_3');
				const [governor] = await tribeca.findGovernorAddress(base.publicKey);
				const smartWallet = await gokiSdk.newSmartWallet({
					owners: [governor],
					threshold: new anchor.BN(1),
					numOwners: 2,
					base
				});
				await smartWallet.tx.confirm();
				const governSdk = new tribeca.GovernWrapper(tribecaSdk);
				const locker = tribeca.getLockerAddress(base.publicKey);
				const govern = await governSdk.createGovernor({
					electorate: locker,
					smartWallet: smartWallet.smartWalletWrapper.key,
					baseKP: base
				});
				await govern.tx.confirm();
				newEvent.set({ name: 'created smart wallet', success: true });
			} catch (err) {
				newEvent.set({ name: `could not create smart wallet`, success: false });
			}
		}
	}

	async function createLocker() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && tribecaSdk) {
			try {
				const mintPk = SURE_MINT;
				const base = getTestKeypairFromSeed(oracleSdk, 'sure_base_3');
				const [governor] = await tribeca.findGovernorAddress(base.publicKey);
				const createLockerRes = await tribecaSdk.createLocker({
					governor: governor,
					govTokenMint: mintPk,
					baseKP: base
				});
				const txRec = await createLockerRes.tx.confirm();
				newEvent.set({ name: 'created sure locker!', status: 'success', tx: txRec.signature });
			} catch (err) {
				const error = err as web3.SendTransactionError;
				newEvent.set({ name: `could not create sure locker`, status: 'error', tx: error.message });
			}
		}
	}
</script>

<div
	class={css`
		position: absolute;
		bottom: 0px;
		left: 0px;
		background: gray;

		display: flex;
		justify-content: flex-start;
		gap: 5;
	`}
>
	<button
		on:click={() => createCustomMint()}
		class={css`
			background: yellow;
			padding: 10px;
		`}>New Mint</button
	>
	<button
		on:click={() => mintToUser()}
		class={css`
			background: yellow;
			padding: 10px;
		`}>Mint 100 SURE</button
	>
	<button
		on:click={() => createSmartWallet()}
		class={css`
			background: yellow;
			padding: 10px;
		`}>Create smart wallet</button
	>

	<button
		on:click={() => createLocker()}
		class={css`
			background: yellow;
			padding: 10px;
		`}>Create Sure locker</button
	>

	<button
		on:click={() =>
			newEvent.set({
				name: Date.now().toString(),
				message:
					'this dsds d sd dsd dsds s d s is an awesome message thea tekndjksdlksfs sinfso ldj dolfd snoif s',
				status: 'success',

				tx: '5jQSFQH2toEQv86DC9KMg7afrWpVsaksaiRsKayZC5jL2GuG13xKHVrZZtyLzGThZfNriocZ765eN57XRMsijXeB'
			})}
		class={css`
			background: yellow;
			padding: 10px;
		`}>Create Event</button
	>
</div>

<style lang="scss">
</style>
