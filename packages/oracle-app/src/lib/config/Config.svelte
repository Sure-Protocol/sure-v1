<script lang="ts">
	import StatBox from '$lib/box/StatBox.svelte';
	import MainButton from '$lib/button/MainButton.svelte';
	import {
		calculateAmountInDecimals,
		calculateAmountInGivenDecimals,
		countdownFromUnix,
		unixSecondsToReadableString
	} from '$lib/utils';
	import { configState, globalStore, newEvent, oneDivXToFloat, tokenState } from '$stores/index';
	import { css } from '@emotion/css';
	import { SURE_MINT } from '@surec/oracle';
	import * as spl from './../../../../../node_modules/@solana/spl-token';
	import expand from '$assets/icons/expandMore.svg';

	let isOpen = false;

	async function createConfig() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk) {
			try {
				const config = await oracleSdk.config().initializeOracleConfig({
					protocolAuthority: oracleSdk.provider.walletKey,
					tokenMint: SURE_MINT
				});
				const txRec = await config.confirm();
				newEvent.set({
					name: 'created config for oracle!',
					tx: txRec.signature,
					status: 'success'
				});
			} catch (err) {
				newEvent.set({
					name: 'could not create config',
					message: err as string,
					status: 'error'
				});
			}
		}
	}
</script>

<div class="action-container--width-s action-container--padding-h0 ">
	<div
		class={css`
			display: flex;
			flex-direction: column;
			gap: 10px;
			width: 100%;
			color: white;
		`}
	>
		<div
			class={css`
				display: flex;
				justify-content: space-between;
			`}
		>
			<h3 class="h3--white">{`Oracle configuration`}</h3>
			<button
				class={css`
					background: transparent;
					border: none;
					cursor: pointer;
				`}
				on:click={() => {
					isOpen = !isOpen;
				}}
			>
				<img width="50px" src={expand} alt="expand oracle configuration" />
			</button>
		</div>
		{#if isOpen}
			<div>
				{#if $configState.config}
					<div
						class={css`
							display: flex;
							gap: 10px;
							flex-wrap: wrap;
						`}
					>
						<StatBox
							title={'voting period'}
							value={unixSecondsToReadableString(
								$configState.config.votingLengthSeconds.toNumber()
							)}
						/>
						<StatBox
							title={'reveal period'}
							value={unixSecondsToReadableString(
								$configState.config.revealLengthSeconds.toNumber()
							)}
						/>
						<StatBox
							title={'required votes'}
							value={calculateAmountInGivenDecimals(
								$configState.config.defaultRequiredVotes,
								$tokenState.mintDecimals
							).toString()}
						/>
						<StatBox
							title={'min proposal stake'}
							value={calculateAmountInGivenDecimals(
								$configState.config.minimumProposalStake,
								$tokenState.mintDecimals
							).toString()}
						/>
						<StatBox
							title={'% vote stake rate '}
							value={100 * oneDivXToFloat($configState.config.voteStakeRate)}
						/>
						<StatBox
							title={'% protocol fee rate'}
							value={100 * oneDivXToFloat($configState.config.protocolFeeRate)}
						/>
					</div>
				{:else if $configState.isLoading}
					<p>Loading config ...</p>
				{:else if $configState.loadingFailed}
					<p>failed to load config</p>
				{:else}
					<p>config does not exist</p>
					<MainButton title="create config" click={() => createConfig()} />
				{/if}
			</div>
		{/if}
	</div>
</div>
