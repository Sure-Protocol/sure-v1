<script lang="ts">
	import { css } from '@emotion/css';
	import { globalStore, newEvent } from './../../../stores/global';
	import type { ProposalType } from '@surec/oracle';
	import type { ProgramAccount } from '@project-serum/anchor';

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
				await voteTx.confirm();
				newEvent.set({ name: 'successfully collect user vote rewards' });
			} catch (err) {
				console.log('failed to collect vote rewards: cause: ', err);
				newEvent.set({ name: 'failed to collect vote rewards' });
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
			width: 5rem;
		`}
	>
		<label
			class={css`
				margin-bottom: 2px;
			`}
			for="userVote">Collect Reward</label
		>
		<button class={'vote-button'} aria-disabled="true">
			<p class="p p--small p--white text--margin-vertical__0">Collect Reward</p>
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
