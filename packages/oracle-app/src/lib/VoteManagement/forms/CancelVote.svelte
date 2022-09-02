<script lang="ts">
	import { css } from '@emotion/css';
	import { globalStore, newEvent } from '$stores/index';
	import type { ProposalType } from '@surec/oracle';
	import type { ProgramAccount } from '@project-serum/anchor';
	import MainButton from '$lib/button/MainButton.svelte';
	import type { SendTransactionError } from '@solana/web3.js';

	export let proposal: ProgramAccount<ProposalType>;

	async function cancelVote() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && proposal) {
			try {
				const [voteAccount] = await oracleSdk.pda.findVoteAccount({
					proposal: proposal.publicKey,
					voter: oracleSdk.provider.wallet.publicKey
				});
				const voteTx = await oracleSdk.vote().cancelVote({
					voteAccount
				});
				const txRec = await voteTx.confirm();
				newEvent.set({
					name: 'successfully cancelled user vote',
					status: 'success',
					tx: txRec.signature
				});
			} catch (err) {
				const error = err as SendTransactionError;
				console.log('failed to cancel vote: cause: ', err);
				newEvent.set({ name: 'failed to cancel vote', status: 'error', tx: error.message });
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
	on:submit|preventDefault={cancelVote}
>
	<div
		class={css`
			display: flex;
			flex-direction: column;
		`}
	>
		<label
			class={css`
				margin-bottom: 2px;
			`}
			for="userVote">Cancel Vote</label
		>
		<MainButton title={'Cancel'} type={'submit'} />
	</div>
</form>

<style lang="scss">
	.vote-button {
		padding-left: 10px;
		padding-right: 10px;
		padding-top: 2px;
		padding-bottom: 2px;
		height: 2rem;
		border: white 1px solid;
		border-radius: 10px;
		background-color: transparent;
		cursor: pointer;

		&:hover {
			border: #f50093 1px solid;
		}
	}
</style>
