<script lang="ts">
	import { css } from '@emotion/css';
	import { globalStore, newEvent } from './../../../stores/global';
	import type { ProposalType } from '@surec/oracle';
	import { BN, type ProgramAccount } from '@project-serum/anchor';
	import { saveSalt } from '$utils';
	import MainButton from '$lib/button/MainButton.svelte';
	import type { SendTransactionError } from '@solana/web3.js';

	export let proposal: ProgramAccount<ProposalType>;

	let formValues = {
		vote: 0.0
	};

	async function updateVote() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && proposal) {
			const userVoteQ32 = new BN(Math.floor(formValues.vote * Math.pow(2, 32)));
			try {
				const voteTx = await oracleSdk.vote().updateVote({
					vote: userVoteQ32,
					proposal: proposal.publicKey
				});
				const txRec = await voteTx.transactionEnvelope.confirm();
				saveSalt(voteTx.salt, proposal.account.name);
				newEvent.set({
					name: 'successfully updated user vote',
					status: 'success',
					tx: txRec.signature
				});
			} catch (err) {
				const error = err as SendTransactionError;
				newEvent.set({
					name: 'failed to update user vote',
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
	on:submit|preventDefault={updateVote}
>
	<div
		class={css`
			display: flex;
			flex-direction: column;
			width: 5rem;
		`}
	>
		<label
			class={css`
				margin-bottom: 2px;
			`}
			for="userVote">Update Vote</label
		>
		<input
			bind:value={formValues.vote}
			id="userVote"
			name="userVote"
			type="decimal"
			class="input-text-field__centered"
		/>
	</div>
	<div
		class={css`
			display: flex;
			justify-content: center;
			align-items: center;
		`}
	>
		<MainButton title={'Update'} type={'submit'} />
	</div>
</form>

<style lang="scss">
</style>
