<script lang="ts">
	import { css } from '@emotion/css';
	import { BN } from '@project-serum/anchor';
	import { onDestroy } from 'svelte';
	import type { ProposalType, VoteStatus, ProposalStatus, VoteAccount } from '@surec/oracle';
	import { getProposalStatus, SureOracleSDK, getVoteStatus } from '@surec/oracle';
	import { findEscrowAddress } from '@tribecahq/tribeca-sdk';
	import {
		getLockerSdk,
		unixToReadable,
		countdownFromUnix,
		isInFuture,
		getNextDeadline,
		saveSalt
	} from '$utils';
	import { selectedProposal, globalStore, newEvent } from './../../stores/global';
	import { SURE_MINT_DEV } from './../constants';
	import type { ProgramAccount } from '@saberhq/token-utils';
	import { Steps } from 'svelte-steps';
	import CreateProposal from './../CreateProposal.svelte';
	import UpdateVote from './forms/UpdateVote.svelte';
	import CancelVote from './forms/CancelVote.svelte';
	import RevealVote from './forms/RevealVote.svelte';
	import CollectRewards from './forms/CollectRewards.svelte';

	let steps: { status: VoteStatus; text: string }[] = [
		{ status: 'Voting', text: 'Voting' },
		{ status: 'Reveal vote', text: 'Reveal vote' },
		{ status: 'Calculate Reward', text: 'Calculate Reward' },
		{ status: 'Collect Reward', text: 'Collect Reward' },
		{ status: 'Failed ', text: 'Failed' }
	];
	let currentStep: number = 0;

	let timer: NodeJS.Timer;
	let countdown: string = '^';
	let vote: VoteAccount | null = null;
	let voteValues = {
		userVote: 0.0
	};

	let proposal: ProgramAccount<ProposalType> | undefined = undefined;

	selectedProposal.subscribe(async (p) => {
		proposal = p;

		// select vote
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && p) {
			const [votePda] = SureOracleSDK.pda().findVoteAccount({
				proposal: p.publicKey,
				voter: oracleSdk?.provider.wallet.publicKey
			});
			try {
				const voteAccount = await oracleSdk.program.account.voteAccount.fetch(votePda);
				vote = voteAccount;
			} catch {
				vote = null;
			}

			// update steps
			currentStep = steps.findIndex((val) => val.text == getVoteStatus(p?.account));
		}

		timer = setInterval(() => {
			let voteEndTime = proposal?.account.voteEndAt;
			let revealEndTime = proposal?.account.voteEndRevealAt;
			const endTime = getNextDeadline([voteEndTime, revealEndTime]);
			let updatedText = steps[currentStep]?.status.toString() ?? 'PH';
			if (isInFuture(endTime)) {
				countdown = countdownFromUnix(endTime);
				updatedText = `${updatedText} ${countdown.toString()}`;
			}
			steps[currentStep] = {
				...steps[currentStep],
				text: updatedText
			};
		}, 1000);
	});

	async function voteOnProposal() {
		const oracleSdk = $globalStore.oracleSDK;
		const lockerSdk = await getLockerSdk(oracleSdk);
		if (oracleSdk && proposal && lockerSdk?.locker) {
			const userVoteQ32 = new BN(Math.floor(voteValues.userVote * Math.pow(2, 32)));
			try {
				const [escrowKey] = await findEscrowAddress(
					lockerSdk.locker,
					oracleSdk.provider.wallet.publicKey
				);
				const voteTx = await oracleSdk.vote().submitVote({
					vote: userVoteQ32,
					mint: SURE_MINT_DEV,
					proposal: proposal.publicKey,
					locker: lockerSdk?.locker,
					userEscrow: escrowKey
				});
				await voteTx.transactionEnvelope.confirm();
				saveSalt(voteTx.salt, proposal.account.name);
				newEvent.set({ name: 'successfully submitted user vote vote' });
			} catch (err) {
				console.log('failed to submit vote: cause: ', err);
				newEvent.set({ name: 'failed to submit vote' });
			}
		}
	}

	onDestroy(() => {
		clearInterval(timer);
	});
</script>

<div class="action-container--width-s action-container--padding-h0 ">
	<div
		class={css`
			width: 100%;
			color: white;
		`}
	>
		<h3 class="h3--white">{`Vote management`}</h3>
		{#if proposal !== undefined}
			<p>{proposal.account.name}</p>
			{#if steps[currentStep].status == 'Failed'}
				<Steps primary={'#d4100b'} current={currentStep} size="1rem" line="1px" {steps} />
			{:else}
				<Steps primary={'#d4100b'} current={currentStep} size="1rem" line="1px" {steps} />
			{/if}

			{#if vote}
				<div
					class={css`
						display: flex;
						flex-direction: column;
						justify-content: space-between;
						padding: 10px;
					`}
				>
					<h3 class=" h3 h3--white">Stats</h3>
					<div
						class={css`
							display: flex;
							justify-content: flex-start;
							gap: 10px;
							margin-bottom: 5px;
						`}
					>
						<div class="info-box">
							<p class="p p--small">{`Vote power - ${vote.votePower}`}</p>
						</div>
						<div class="info-box"><p class="p p--small">{`Rewards -${vote.earnedRewards}`}</p></div>
						<div class="info-box"><p class="p p--small">{`Revealed vote - ${vote.vote}`}</p></div>
						{#if vote.locked}
							<div class="info-box"><p class="p p--small">{`Locked`}</p></div>
						{/if}
					</div>
					<h3 class=" h3 h3--white">Actions</h3>
					{#if steps[currentStep].status == 'Voting'}
						{#if vote}
							<div>
								<UpdateVote {proposal} />
							</div>
							<div>
								<CancelVote {proposal} />
							</div>
						{:else}
							<form on:submit|preventDefault={voteOnProposal}>
								<div
									class={css`
										display: flex;
										flex-direction: column;
										width: 5rem;
									`}
								>
									<span>Submit Vote</span>
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
						{/if}
					{:else if steps[currentStep].status == 'Reveal vote'}
						<div>
							<RevealVote {proposal} />
						</div>
					{:else if steps[currentStep].status == 'Calculate Reward'}
						<div>Calculate reward</div>
					{:else if steps[currentStep].status == 'Collect Reward'}
						<div>
							<CollectRewards {proposal} />
						</div>
					{:else}
						<div>
							<CollectRewards {proposal} />
						</div>
					{/if}
				</div>
			{/if}
		{:else}
			<p>Pick a proposal...</p>
		{/if}
	</div>
</div>

<style lang="scss">
	.info-box {
		display: flex;
		border: white 1px solid;
		border-radius: 10px;
		padding-left: 10px;
		padding-right: 10px;
		justify-content: center;
		align-items: center;
	}
</style>
