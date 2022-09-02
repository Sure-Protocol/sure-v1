<script lang="ts">
	import { css } from '@emotion/css';
	import { globalStore, newEvent } from '$stores/index';
	import type { ProposalType } from '@surec/oracle';
	import { BN, type ProgramAccount } from '@project-serum/anchor';
	import { saveSalt } from '$lib/utils';
	import MainButton from '$lib/button/MainButton.svelte';
	import type { SendTransactionError } from '@solana/web3.js';
	import SingleInput from '$lib/input/SingleInput.svelte';

	export let proposal: ProgramAccount<ProposalType>;
	export let title: string = 'Update vote';
	export let description: string | undefined = undefined;
	export let buttonTitle: string = 'Update';
	export let submitAction: () => void = updateVote;

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
	on:submit|preventDefault={submitAction}
>
	<div
		class={css`
			display: flex;
			flex-direction: row;
			gap: 10px;
			justify-content: center;
			align-items: flex-end;
		`}
	>
		<SingleInput {title} {description}>
			<input
				slot="input"
				bind:value={formValues.vote}
				id="userVote"
				name="userVote"
				type="decimal"
				class="input-text-field__centered"
			/>
		</SingleInput>
		<div
			class={css`
				display: flex;
				justify-content: bottom;
				align-items: bottom;
			`}
		>
			<MainButton title={buttonTitle} type={'submit'} />
		</div>
	</div>
</form>

<style lang="scss">
</style>
