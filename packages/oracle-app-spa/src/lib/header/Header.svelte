<script lang="ts">
	import { css } from '@emotion/css';
	import logo from '$assets/icons/sureLogo.svg';
	import { onMount } from 'svelte';
	import { clusterApiUrl } from '@solana/web3.js';
	import {
		workSpace,
		WalletProvider,
		WalletMultiButton,
		ConnectionProvider,
	} from '@svelte-on-solana/wallet-adapter-ui';
	import type { Adapter } from '@sveltejs/kit';
	import {
		startLoading,
		loadingState,
		globalStore,
		hydrateProposals,
		hydrateConfig,
		loadingFailed,
		loadingSuccessful,
		hydrateTokenState,
	} from '$stores/index';

	const localStorageKey = 'walletAdapter';
	const network = clusterApiUrl('devnet'); // localhost or mainnet

	let wallets: Adapter[];

	let time: number = 0;
	let timeUnix: number = 0;

	onMount(async () => {
		const {
			PhantomWalletAdapter,
			SlopeWalletAdapter,
			SolflareWalletAdapter,
			SolletExtensionWalletAdapter,
			TorusWalletAdapter,
		} = await import('@solana/wallet-adapter-wallets');

		const walletsMap = [
			new PhantomWalletAdapter(),
			new SlopeWalletAdapter(),
			new SolflareWalletAdapter(),
			new SolletExtensionWalletAdapter(),
			new TorusWalletAdapter(),
		];

		wallets = walletsMap;

		setInterval(() => {
			time = new Date().toLocaleString();
			timeUnix = Math.floor(Date.now() / 1000);
		}, 1000);
	});

	globalStore.subscribe((store) => {
		if (store.oracleSDK && $loadingState.refresh) {
			startLoading();
			try {
				Promise.all([
					hydrateProposals(store.oracleSDK),
					hydrateConfig(store.oracleSDK),
					hydrateTokenState(store.oracleSDK),
				])
					.then(() => loadingSuccessful())
					.catch(() => {
						loadingFailed();
					});
			} catch {
				loadingFailed();
			}
		}
	});

	/// fetch necessary data
	loadingState.subscribe(async (val) => {
		if (val.refresh && !val.isLoading) {
			const oracleSdk = $globalStore.oracleSDK;
			if (oracleSdk) {
				startLoading();
				try {
					await hydrateProposals(oracleSdk);
					await hydrateConfig(oracleSdk);
					await hydrateTokenState(oracleSdk);
					loadingSuccessful();
				} catch {
					loadingFailed();
				}
			}
		}
	});
</script>

<header>
	<div class="corner">
		<a href="sure.claims">
			<img src={'assets/icons/sureLogo.svg'} alt="Sure protocol" />
		</a>
	</div>
	<div>
		<div
			class={css`
				color: green;
			`}
		>
			{`${time} | ${timeUnix}`}
		</div>
	</div>

	<WalletProvider {localStorageKey} {wallets} autoConnect />
	<ConnectionProvider {network} />
	<WalletMultiButton />
</header>

<style>
	header {
		display: flex;
		justify-content: space-between;
		padding-top: 3rem;
		padding-bottom: 3rem;
		padding: 1rem;
	}

	.corner {
		width: 3em;
		height: 3em;
	}

	.corner a {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
	}

	.corner img {
		width: 2em;
		height: 2em;
		object-fit: contain;
	}

	nav {
		display: flex;
		justify-content: center;
		--background: rgba(255, 255, 255, 0.7);
	}

	svg {
		width: 2em;
		height: 3em;
		display: block;
	}

	path {
		fill: var(--background);
	}

	ul {
		position: relative;
		padding: 0;
		margin: 0;
		height: 3em;
		display: flex;
		justify-content: center;
		align-items: center;
		list-style: none;
		background: var(--background);
		background-size: contain;
	}

	li {
		position: relative;
		height: 100%;
	}

	li.active::before {
		--size: 6px;
		content: '';
		width: 0;
		height: 0;
		position: absolute;
		top: 0;
		left: calc(50% - var(--size));
		border: var(--size) solid transparent;
		border-top: var(--size) solid var(--accent-color);
	}

	nav a {
		display: flex;
		height: 100%;
		align-items: center;
		padding: 0 1em;
		color: var(--heading-color);
		font-weight: 700;
		font-size: 0.8rem;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		text-decoration: none;
		transition: color 0.2s linear;
	}

	a:hover {
		color: var(--accent-color);
	}
</style>