<script lang="ts">
	import { css } from '@emotion/css';
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
	let sureTokens = new anchor.BN(0);
	let veSureAmount = new anchor.BN(0);

	let values = {
		amount: 1,
		days: 365
	};

	// send to utils

	async function getVeSureAmount() {
		const oracleSdk = $globalStore.oracleSDK;
		const lockerSdk = await getLockerSdk($globalStore.oracleSDK);
		if (lockerSdk && oracleSdk) {
			try {
				const escrow = await lockerSdk.fetchEscrowByAuthority();
				const newVeSureAmount = await calculateAmountInDecimals(oracleSdk, escrow.amount);
				if (newVeSureAmount) {
					veSureAmount = newVeSureAmount;
				}
			} catch {}
		}
	}

	async function getSureAmount() {
		const oracleSdk = $globalStore.oracleSDK;
		const newSureAmount = await calculateAccountBalanceInDecimals(oracleSdk);
		if (newSureAmount) {
			sureTokens = newSureAmount;
		}
	}

	async function lockSureTokens() {
		const oracleSdk = $globalStore.oracleSDK;
		const lockerSdk = await getLockerSdk(oracleSdk);
		if (lockerSdk && oracleSdk) {
			try {
				const lockAmount = await calculateFullAmount(oracleSdk, new anchor.BN(values.amount));
				if (lockAmount) {
					newEvent.set({ name: `lock ${lockAmount} for ${values.days} days ` });
					const lockTokensTx = await lockerSdk.lockTokens({
						amount: lockAmount,
						duration: daysToSecond(values.days)
					});
					await lockTokensTx.confirm();
					newEvent.set({ name: `successfully locked ${lockAmount} for ${values.days} ` });
				}
			} catch (err) {
				console.log('could not lock tokens. Cause: ', err);
				newEvent.set({ name: 'could not lock tokens.' });
			}
		}
	}

	onMount(async () => {
		getSureAmount();
		getVeSureAmount();
	});

	wallet_adapter.walletStore.subscribe(async (value) => {
		getSureAmount();
		getVeSureAmount();
	});
</script>

<div class="action-container--width-s action-container--padding-h0 ">
	<form
		on:submit|preventDefault={lockSureTokens}
		class={css`
			width: 100%;
			color: white;
		`}
	>
		<h3 class="h3--white">Top up veSure</h3>
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
				<input
					bind:value={values.amount}
					type="number"
					name="lockAmount"
					id="lockAmount"
					class="input-number-field--padding-m"
				/>
			</div>

			<div
				class={css`
					width: fit-content;
				`}
			>
				<label for="lockDays">Days to lock</label>
				<input
					bind:value={values.days}
					type="number"
					name="lockDays"
					id="lockDays"
					class="input-number-field--padding-m"
				/>
			</div>
		</div>

		<p>{`Amount of sure: ${sureTokens}`}</p>
		<p>{`Amount of veSure: ${veSureAmount}`}</p>
		<button type="submit">Lock</button>
	</form>
</div>
