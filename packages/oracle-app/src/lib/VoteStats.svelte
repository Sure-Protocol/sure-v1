<script lang="ts">
	import { css } from '@emotion/css';
	import { BN } from '@project-serum/anchor';
	import type { AnchorAccount } from '@saberhq/anchor-contrib/dist/cjs/utils/accounts';
	import type { ProposalType } from '@surec/oracle';
	import { findEscrowAddress } from '@tribecahq/tribeca-sdk';
	import { getLockerSdk } from '$utils';
	import { selectedProposal, globalStore, newEvent } from './../stores/global';
	import { SURE_MINT_DEV } from './constants';
	import type { ProgramAccount } from '@saberhq/token-utils';

	let voteValues = {
		userVote: 0.0
	};

	let proposal: ProgramAccount<ProposalType> | undefined = undefined;

	selectedProposal.subscribe((p) => {
		console.log('selected proposal');
		proposal = p;
	});

	async function voteOnProposal() {
		const oracleSdk = $globalStore.oracleSDK;
		const lockerSdk = await getLockerSdk(oracleSdk);
		if (oracleSdk && proposal && lockerSdk?.locker) {
			const userVoteQ32 = voteValues.userVote;
			try {
				const [escrowKey] = await findEscrowAddress(
					lockerSdk.locker,
					oracleSdk.provider.wallet.publicKey
				);
				const voteTx = await oracleSdk.vote().submitVote({
					vote: new BN(voteValues.userVote),
					mint: SURE_MINT_DEV,
					proposal: proposal.publicKey,
					locker: lockerSdk?.locker,
					userEscrow: escrowKey
				});
				await voteTx.transactionEnvelope.confirm();
				newEvent.set({ name: 'successfully submitted user vote vote' });
			} catch (err) {
				console.log('failed to submit vote: cause: ', err);
				newEvent.set({ name: 'failed to submit vote' });
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
		<h3 class="h3--white">Vote stats</h3>
		{#if proposal !== undefined}
			<form on:submit|preventDefault={voteOnProposal}>
				<div>
					<span>Answer</span>
					<input
						bind:value={voteValues.userVote}
						id="userVote"
						name="userVote"
						type="decimal"
						class="input-text-field"
					/>
				</div>
				<button>Vote</button>
			</form>
		{:else}
			<p>Pick a proposal...</p>
		{/if}
	</div>
</div>
