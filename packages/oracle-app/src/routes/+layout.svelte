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

	let showProposal = false;

	wallet_adapter.walletStore.subscribe((value) => {
		console.log('<<<>>>> wallet adapter');
		let connection = new web3.Connection(web3.clusterApiUrl('devnet'));
		if (value.wallet?.publicKey != null) {
			const oracleProvider = solana_contrib.SolanaProvider.init({
				connection,
				wallet: value.wallet,
				opts: { skipPreflight: true }
			});

			const oracleSdk = oracle.SureOracleSDK.init({ provider: oracleProvider });
			$globalStore.oracleSDK = oracleSdk;
			$globalStore.walletPk = value.wallet.publicKey;
			$globalStore.wallet = value.wallet;
			$globalStore.provider = oracleProvider;
		}
	});

	onMount(() => {
		createProposalState.subscribe((val) => {
			showProposal = val;
		});
	});
</script>

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
	{#if process.env.SURE_ENV == 'dev'}
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
