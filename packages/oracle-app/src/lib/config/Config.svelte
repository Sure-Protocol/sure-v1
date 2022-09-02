<script lang="ts">
	import StatBox from '$lib/box/StatBox.svelte';
	import MainButton from '$lib/button/MainButton.svelte';
	import {
		calculateAmountInDecimals,
		countdownFromUnix,
		unixSecondsToReadableString
	} from '$lib/utils';
	import { configState, globalStore, newEvent } from '$stores/index';
	import { css } from '@emotion/css';
	import { SURE_MINT } from '@surec/oracle';

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
			width: 100%;
			color: white;
		`}
	>
		<h3 class="h3--white">{`Oracle Configuration`}</h3>
		<p>Create or configure the oracle</p>
		{#if $configState.config}
			<div
				class={css`
					display: flex;
				`}
			>
				<StatBox
					title={'Voting Length'}
					value={unixSecondsToReadableString($configState.config.votingLengthSeconds.toNumber())}
				/>
				<StatBox
					title={'Reveal Length'}
					value={unixSecondsToReadableString($configState.config.revealLengthSeconds.toNumber())}
				/>
				<StatBox title={'Required votes'} value={$configState.config.defaultRequiredVotes} />
				<StatBox
					title={'Minimum proposal stake'}
					value={$configState.config.minimumProposalStake.toString()}
				/>
				<StatBox title={'Vote stake rate'} value={$configState.config.voteStakeRate} />
				<StatBox title={'protocol fee rate'} value={$configState.config.protocolFeeRate} />
			</div>
		{:else}
			<p>Could not load config</p>
			<MainButton title="create config" click={() => createConfig()} />
		{/if}
	</div>
</div>
