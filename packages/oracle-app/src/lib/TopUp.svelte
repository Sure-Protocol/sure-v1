<script lang="ts">
	import { css, cx } from '@emotion/css';
	import * as anchor from '@project-serum/anchor';
	import * as spl from './../../node_modules/@solana/spl-token';
	import * as web3 from '@solana/spl-token';
	import * as tribeca from '@tribecahq/tribeca-sdk';
	import { onMount } from 'svelte';
	import { globalStore, newEvent } from '../stores/global';
	import { SURE_MINT_DEV } from './constants';
	import * as wallet_adapter from '@svelte-on-solana/wallet-adapter-core';
	import type { AnchorAccount } from '@saberhq/anchor-contrib/dist/cjs/utils/accounts';
	import {
		getLockerSdk,
		daysToSecond,
		calculateFullAmount,
		calculateAccountBalanceInDecimals,
		calculateAmountInDecimals
	} from './../utils/';
	import MainButton from './button/MainButton.svelte';
	import type { SendTransactionError } from '@solana/web3.js';
	import TypeInputAmount from './input/TypeInputAmount.svelte';
	import Amount from './text/Amount.svelte';
	let sureTokens = '__';
	let veSureAmount = '__';

	let values = {
		amount: undefined,
		days: undefined
	};
	let loadingData = true;

	// send to utils

	async function getVeSureAmount() {
		const oracleSdk = $globalStore.oracleSDK;
		const lockerSdk = await getLockerSdk($globalStore.oracleSDK);
		if (lockerSdk && oracleSdk) {
			try {
				const escrow = await lockerSdk.fetchEscrowByAuthority();
				const newVeSureAmount = await calculateAmountInDecimals(oracleSdk, escrow.amount);
				if (newVeSureAmount) {
					veSureAmount = newVeSureAmount.toString();
				}
			} catch {
				throw new Error();
			}
		} else {
			throw new Error();
		}
	}

	async function getSureAmount() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk) {
			const newSureAmount = await calculateAccountBalanceInDecimals(oracleSdk);
			if (newSureAmount) {
				sureTokens = newSureAmount.toString();
			}
		} else {
			throw new Error();
		}
	}

	async function lockSureTokens() {
		const oracleSdk = $globalStore.oracleSDK;
		const lockerSdk = await getLockerSdk(oracleSdk);
		if (lockerSdk && oracleSdk) {
			try {
				const lockAmount = await calculateFullAmount(oracleSdk, new anchor.BN(values.amount));
				if (lockAmount) {
					newEvent.set({ name: `lock ${lockAmount} for ${values.days} days `, status: 'info' });
					const lockTokensTx = await lockerSdk.lockTokens({
						amount: lockAmount,
						duration: daysToSecond(values.days)
					});
					const txRes = await lockTokensTx.confirm();
					newEvent.set({
						name: `successfully locked ${lockAmount} for ${values.days} `,
						status: 'success',
						tx: txRes.signature
					});
				}
			} catch (err) {
				const error = err as SendTransactionError;
				console.log('could not lock tokens. Cause: ', err);
				newEvent.set({
					name: 'could not lock tokens.',
					message: error.message,
					status: 'error'
				});
			}
		}
	}

	onMount(async () => {
		loadingData = true;
		Promise.all([getSureAmount(), getVeSureAmount()])
			.then(() => {
				loadingData = false;
			})
			.catch(() => {
				loadingData = true;
			});
	});

	wallet_adapter.walletStore.subscribe(async (value) => {
		loadingData = true;
		Promise.all([getSureAmount(), getVeSureAmount()])
			.then(() => {
				loadingData = false;
			})
			.catch(() => {
				loadingData = true;
			});
	});
</script>

<div class="action-container--width-s action-container--padding-h0 ">
	<form
		on:submit|preventDefault={lockSureTokens}
		class={css`
			width: 100%;
			color: white;
			display: flex;
			flex-direction: column;
			gap: 1rem;
			padding-bottom: 1rem;
		`}
	>
		<h3 class="h3--white">Top up veSure</h3>
		<p class="p text--margin-vertical__0">
			.by locking $sure your receive veSure which can be used to vote on proposals and participate
			in governance
		</p>
		<div
			class={css`
				display: flex;
				flex-direction: row;
				justify-content: space-around;
			`}
		>
			<div
				class={css`
					width: fit-content;
				`}
			>
				<label for="lockAmount">Amount to lock</label>
				<TypeInputAmount bind:value={values.amount} valueType={'$sure'} />
			</div>

			<div
				class={css`
					width: fit-content;
				`}
			>
				<label for="lockDays">Days to lock</label>
				<TypeInputAmount bind:value={values.days} valueType={'days'} />
			</div>
		</div>
		<div
			class={css`
				display: flex;
				justify-content: center;
				gap: 5rem;
			`}
		>
			<Amount title="$sure" amount={sureTokens} loading={loadingData} />
			<Amount title="veSure" amount={veSureAmount} loading={loadingData} />
		</div>
		<div
			class={css`
				display: flex;
				justify-content: center;
			`}
		>
			<MainButton title={'Lock'} type={'submit'} />
		</div>
	</form>
</div>
