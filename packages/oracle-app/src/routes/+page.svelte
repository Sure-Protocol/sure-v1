<script lang="ts">
	import { css } from '@emotion/css';
	import ProposalList from '$lib/ProposalList.svelte';
	import * as wallet_adapter from '@svelte-on-solana/wallet-adapter-core';
	import * as oracle from '@surec/oracle';
	import * as web3 from '@solana/web3.js';
	import * as solana_contrib from '@saberhq/solana-contrib';

	import { globalStore } from './../stores/global';
	import TopUp from '$lib/TopUp.svelte';

	wallet_adapter.walletStore.subscribe((value) => {
		let connection = new web3.Connection(web3.clusterApiUrl('devnet'));
		if (value.wallet?.publicKey != null) {
			console.log('wallet:  ', value.wallet.publicKey);
			const oracleProvider = solana_contrib.SolanaProvider.init({
				connection,
				wallet: value.wallet,
				opts: { skipPreflight: true }
			});

			const oracleSDK = oracle.SureOracleSDK.init({ provider: oracleProvider });
			$globalStore.oracleSDK = oracleSDK;
			$globalStore.walletPk = value.wallet.publicKey;
			$globalStore.wallet = value.wallet;
			$globalStore.provider = oracleProvider;
		}
	});
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<div class="action-container action-container--transparent action-container--width-full">
	<div class="action-container-inner">
		<div class="action-container-inner-content--row">
			<div class="action-container-inner-content--item">
				<div class="action-container--width-l">
					<div class="action-container-inner">
						<h2 class="h3--white">Votes</h2>
						<div>
							<input type="text" class="input-text-field" placeholder="search votes" />
							<button>Filter</button>
						</div>
						<ProposalList />
					</div>
				</div>
			</div>
			<div class="action-container-inner-content--item">
				<div class="action-container-inner-content">
					<div class="action-container-inner-content--item">
						<TopUp />
					</div>
					<div class="action-container-inner-content--item">
						<div class="action-container--width-s action-container--padding-h0 ">
							<div
								class={css`
									width: 100%;
									color: white;
								`}
							>
								<h3 class="h3--white">Vote stats</h3>
								<p>Pick a proposal...</p>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>

<style lang="scss" global>
	@import '../../../sure-static/styles/index.scss';

	progress {
		border-radius: 0px;
		width: 80%;
		height: 10px;
		box-shadow: 1px 1px 4px rgba(0, 0, 0, 0.2);
	}
	progress::-webkit-progress-bar {
		background-color: white;
		border-radius: 0px;
	}
	progress::-webkit-progress-value {
		background-color: $sure-pink;
		//border-radius: 7px;
		box-shadow: 1px 1px 1px 1px rgba(0, 0, 0, 0.8);
	}
	progress::-moz-progress-bar {
		/* style rules */
	}

	.voting-status {
		border: $sure-pink 1px solid;
		border-radius: 10px;
		padding-left: 10px;
		padding-right: 10px;
		padding-top: 5px;
		padding-bottom: 5px;
	}

	section {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		flex: 1;
	}

	h1 {
		width: 100%;
	}
</style>
