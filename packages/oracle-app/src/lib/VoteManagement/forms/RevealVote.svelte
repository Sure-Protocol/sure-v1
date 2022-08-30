<script lang="ts">
	import { css } from '@emotion/css';
	import { globalStore, newEvent } from './../../../stores/global';
	import type { ProposalType } from '@surec/oracle';
	import { BN, type ProgramAccount } from '@project-serum/anchor';
	import { getSalt } from '$utils';

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
				await voteTx.confirm();
				newEvent.set({ name: 'successfully revealed the user vote vote' });
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
	on:submit|preventDefault={revealVote}
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
			for="userVote">Reveal Vote</label
		>
		<input
			bind:value={formValues.vote}
			id="userVote"
			name="userVote"
			type="decimal"
			class="input-text-field__centered"
		/>
		<button class={'vote-button'} aria-disabled="true">
			<p class="p p--small p--white text--margin-vertical__0">Reveal Vote</p>
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
