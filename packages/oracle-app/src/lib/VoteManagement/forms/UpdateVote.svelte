<script lang="ts">
	import { css } from '@emotion/css';
	import { globalStore, newEvent } from './../../../stores/global';
	import type { ProposalType } from '@surec/oracle';
	import { BN, type ProgramAccount } from '@project-serum/anchor';
	import { saveSalt } from '$utils';

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
				await voteTx.transactionEnvelope.confirm();
				saveSalt(voteTx.salt, proposal.account.name);
				newEvent.set({ name: 'successfully update user vote vote' });
			} catch (err) {
				console.log('failed to update vote: cause: ', err);
				newEvent.set({ name: 'failed to update vote' });
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
		<button class={'vote-button'} aria-disabled="true">
			<p class="p p--small p--white text--margin-vertical__0">Update Vote</p>
		</button>
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
