<script lang="ts">
	import Header from '$lib/header/Header.svelte';
	import CreateProposal from '$lib/CreateProposal.svelte';
	import { css } from '@emotion/css';
	import { writable } from 'svelte/store';
	import { globalStore, loadingState } from './../stores/global';
	import { createProposalState, hydrateProposals, proposalsState } from '$stores/index';
	import TestPanel from '$lib/test/TestPanel.svelte';
	import { onMount } from 'svelte';
	import EventStack from '$lib/EventStack.svelte';
	import * as wallet_adapter from '@svelte-on-solana/wallet-adapter-core';
	import * as oracle from '@surec/oracle';
	import * as web3 from '@solana/web3.js';
	import * as solana_contrib from '@saberhq/solana-contrib';
	import { newEvent } from '$stores/index';
	import { getLocalStorage, setLocalStorage } from '@svelte-on-solana/wallet-adapter-core';

	let showProposal = false;
	let testModeActivated = false;
	let keyCombo = '';
	let testMethodsOn = wallet_adapter.walletStore.subscribe((value) => {
		let connection = new web3.Connection(web3.clusterApiUrl('devnet'));
		if (value.wallet?.publicKey != null) {
			const oracleProvider = solana_contrib.SolanaProvider.init({
				connection,
				wallet: value.wallet,
				opts: { skipPreflight: true }
			});

			const oracleSdk = oracle.SureOracleSDK.init({ provider: oracleProvider });
			$globalStore = {
				oracleSDK: oracleSdk,
				walletPk: value.wallet.publicKey,
				wallet: value.wallet,
				provider: oracleProvider
			};
		}
	});

	onMount(() => {
		setInterval(() => {
			keyCombo = '';
			if (getLocalStorage('testMode') == 'on') {
				testModeActivated = true;
			} else {
				testModeActivated = false;
			}
		}, 2000);
	});

	onMount(() => {
		createProposalState.subscribe((val) => {
			showProposal = val;
		});
	});

	const testModeListener = (event: KeyboardEvent) => {
		if (keyCombo.length > 4) {
			keyCombo = '';
		}
		keyCombo = `${keyCombo}${event.key}`;
		if (keyCombo == 'awds') {
			if (getLocalStorage('testMode') == 'on') {
				setLocalStorage('testMode', 'off');
			} else {
				setLocalStorage('testMode', 'on');
			}
		}
	};
</script>

<svelte:window on:keydown={testModeListener} />

<div
	class={css`
		height: 100vh;
	`}
>
	<Header />

	<main class={showProposal ? 'blurred' : ''}>
		<slot />
	</main>
	{#if showProposal}
		<CreateProposal />
	{/if}

	<EventStack />
	{#if testModeActivated}
		<TestPanel />
	{/if}
</div>

<style lang="scss">
	@import '../../../sure-static/styles/index.scss';
	:global {
		body {
			background-color: $sure-black;
			width: auto;
			height: 100vh;
		}
	}
	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 1rem;
		width: 100%;
		height: 100vh;
		margin: 0 auto;
		box-sizing: border-box;
		background-color: $sure-black;
	}

	.blurred {
		filter: blur(10px);
	}

	footer {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		padding: 40px;
		background-color: $sure-black;
	}

	footer a {
		font-weight: bold;
	}

	@media (min-width: 480px) {
		footer {
			padding: 40px 0;
		}
	}
</style>
