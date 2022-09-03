<script lang="ts">
	import { css } from '@emotion/css';
	import { BN } from '@project-serum/anchor';
	import { onDestroy } from 'svelte';
	import {
		type ProposalType,
		type VoteStatus,
		type ProposalStatus,
		type VoteAccount,
		SURE_MINT,
		proposalFailReason
	} from '@surec/oracle';
	import { getProposalStatus, SureOracleSDK, getVoteStatus } from '@surec/oracle';
	import { findEscrowAddress } from '@tribecahq/tribeca-sdk';
	import {
		getLockerSdk,
		unixToReadable,
		countdownFromUnix,
		isInFuture,
		getNextDeadline,
		saveSalt
	} from '$lib/utils';
	import {
		selectedProposal,
		globalStore,
		newEvent,
		tokenState,
		proposalSteps
	} from '$stores/index';
	import type { ProgramAccount } from '@saberhq/token-utils';
	import { Steps } from 'svelte-steps';
	import type { SendTransactionError } from '@solana/web3.js';
	import StatBox from '$lib/box/StatBox.svelte';
	import { to_number } from 'svelte/internal';
	import errorCircle from '$assets/icons/errorCircle.svg';

	let steps: { status: VoteStatus; text: string }[] = [
		{ status: 'Voting', text: 'Voting' },
		{ status: 'Reveal vote', text: 'Reveal vote' },
		{ status: 'Calculate Reward', text: 'Calculate Reward' },
		{ status: 'Collect Reward', text: 'Collect Reward' },
		{ status: 'Failed', text: 'Failed' }
	];
	let currentStep: number = 0;
</script>

<div class="action-container--width-s action-container--padding-h0 ">
	<div
		class={css`
			width: 100%;
			color: white;
			padding-bottom: 1rem;
		`}
	>
		<h3 class="h3--white">{`Proposal status`}</h3>
		{#if $selectedProposal !== undefined}
			<p class="p p--italic">{$selectedProposal.account.name}</p>
			<div
				class={css`
					width: 100%;
					margin-bottom: 1rem;
					margin-top: 2rem;
				`}
			>
				{#if $proposalSteps.steps[$proposalSteps.currentStep].status == 'Failed'}
					<Steps primary={'#d4100b'} current={currentStep} size="1rem" line="1px" {steps} />
				{:else}
					<Steps primary={'#d4100b'} current={currentStep} size="1rem" line="1px" {steps} />
				{/if}
			</div>
			{#if $proposalSteps.steps[$proposalSteps.currentStep].status == 'Failed'}
				<div>
					{#if proposalFailReason($selectedProposal.account) == 'NotEnoughVotes'}
						<div
							class={css`
								display: flex;
								gap: 10px;
								align-items: center;
							`}
						>
							<img src={errorCircle} height={'20px'} alt="failed" />
							<p class="p ">{'the proposal failed to reach quorum'}</p>
						</div>
					{/if}
				</div>
			{/if}
		{:else}
			<p>Pick a proposal to view actions</p>
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
