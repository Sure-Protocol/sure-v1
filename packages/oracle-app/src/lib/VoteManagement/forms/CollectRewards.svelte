<script lang="ts">
	import { css } from '@emotion/css';
	import { globalStore, newEvent } from './../../../stores/global';
	import type { ProposalType } from '@surec/oracle';
	import type { ProgramAccount } from '@project-serum/anchor';
	import MainButton from '$lib/button/MainButton.svelte';
	import type { SendTransactionError } from '@solana/web3.js';

	export let proposal: ProgramAccount<ProposalType>;

	async function collectReward() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && proposal) {
			try {
				const [voteAccount] = await oracleSdk.pda.findVoteAccount({
					proposal: proposal.publicKey,
					voter: oracleSdk.provider.wallet.publicKey
				});
				const voteTx = await oracleSdk.vote().collectRewards({
					voteAccount
				});
				const txRec = await voteTx.confirm();
				newEvent.set({
					name: 'successfully collected vote rewards',
					status: 'success',
					tx: txRec.signature
				});
			} catch (err) {
				const error = err as SendTransactionError;
				console.log('failed to collect vote rewards: ', err);
				newEvent.set({
					name: 'failed to collect vote rewards',
					status: 'error',
					message: error.message
				});
			}
		}
	}
</script>

<form
	class={css`
		display: flex;
		flex-direction: row;
		gap: 10px;
	`}
	on:submit|preventDefault={collectReward}
>
	<div
		class={css`
			display: flex;
			flex-direction: column;
		`}
	>
		<MainButton title={'Collect reward'} type={'submit'} />
	</div>
</form>

<style lang="scss">
</style>
