<script lang="ts">
	import { css } from '@emotion/css';
	import { globalStore, newEvent } from './../../../stores/global';
	import type { ProposalType } from '@surec/oracle';
	import { BN, type ProgramAccount } from '@project-serum/anchor';
	import { getSalt } from '$utils';
	import MainButton from '$lib/button/MainButton.svelte';
	import type { SendTransactionError } from '@solana/web3.js';

	export let proposal: ProgramAccount<ProposalType>;

	let formValues = {
		vote: 0.0
	};

	async function revealVote() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && proposal) {
			const userVoteQ32 = new BN(Math.floor(formValues.vote * Math.pow(2, 32)));
			const salt = getSalt(proposal.account.name);
			const [voteAccount] = oracleSdk.pda.findVoteAccount({
				proposal: proposal.publicKey,
				voter: oracleSdk.provider.walletKey
			});
			try {
				const voteTx = await oracleSdk.vote().revealVote({
					vote: userVoteQ32,
					voteAccount,
					salt
				});
				const txRec = await voteTx.confirm();
				newEvent.set({
					name: 'successfully revealed user vote',
					status: 'success',
					tx: txRec.signature
				});
			} catch (err) {
				const error = err as SendTransactionError;
				newEvent.set({ name: 'failed to reveal vote', status: 'error', message: error.message });
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
	on:submit|preventDefault={revealVote}
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
			for="userVote">Reveal Vote</label
		>
		<input
			bind:value={formValues.vote}
			id="userVote"
			name="userVote"
			type="decimal"
			class="input-text-field__centered"
		/>
		<MainButton title={'Reveal'} type={'submit'} />
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
