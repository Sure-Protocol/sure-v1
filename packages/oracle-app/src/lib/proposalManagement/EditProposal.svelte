<script lang="ts">
	import { css } from '@emotion/css';
	import { BN } from '@project-serum/anchor';
	import { onDestroy } from 'svelte';
	import type { ProposalType, VoteStatus, ProposalStatus, VoteAccount } from '@surec/oracle';
	import { getProposalStatus, SureOracleSDK, getVoteStatus } from '@surec/oracle';
	import { findEscrowAddress } from '@tribecahq/tribeca-sdk';
	import {
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
	import type { SendTransactionError } from '@solana/web3.js';
	import StatBox from '$lib/box/StatBox.svelte';
	import { to_number } from 'svelte/internal';

	let steps: { status: VoteStatus; text: string }[] = [
		{ status: 'Voting', text: 'Voting' },
		{ status: 'Reveal vote', text: 'Reveal vote' },
		{ status: 'Calculate Reward', text: 'Calculate Reward' },
		{ status: 'Collect Reward', text: 'Collect Reward' },
		{ status: 'Failed', text: 'Failed' }
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
		if (p) {
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

	function isOnwerOfSelectedProposal(){
		// if (proposal.account.onwer == )
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
		<h3 class="h3--white">{`Proposal management`}</h3>
		{#if proposal !== undefined}
			<p class="p p--italic">{proposal.account.name}</p>
			<div
				class={css`
					width: 100%;
				`}
			>
				{#if steps[currentStep].status == 'Failed'}
					<Steps primary={'#d4100b'} current={currentStep} size="1rem" line="1px" {steps} />
				{:else}
					<Steps primary={'#d4100b'} current={currentStep} size="1rem" line="1px" {steps} />
				{/if}
			</div>
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
		width: 5rem;
		justify-content: center;
		align-items: center;
	}
</style>
