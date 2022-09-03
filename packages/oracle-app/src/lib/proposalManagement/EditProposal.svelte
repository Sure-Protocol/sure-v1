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
	import { countdownFromUnix, isInFuture, getNextDeadline, saveSalt } from '$lib/utils';
	import { selectedProposal, globalStore, newEvent } from '$stores/index';
	import type { ProgramAccount } from '@saberhq/token-utils';
	import { Steps } from 'svelte-steps';
	import CreateProposal from './../CreateProposal.svelte';
	import { SendTransactionError } from '@solana/web3.js';
	import StatBox from '$lib/box/StatBox.svelte';
	import { to_number } from 'svelte/internal';
	import { calculateAmountInDecimals } from '$lib/utils/money';
	import CollectRewards from '$lib/VoteManagement/forms/CollectRewards.svelte';
	import type { TransactionReceipt } from '@saberhq/solana-contrib';
	import errorCircle from '$assets/icons/errorCircle.svg';

	let steps: { status: VoteStatus; text: string }[] = [
		{ status: 'Voting', text: 'Voting' },
		{ status: 'Reveal vote', text: 'Reveal vote' },
		{ status: 'Calculate Reward', text: 'Calculate Reward' },
		{ status: 'Collect Reward', text: 'Collect Reward' },
		{ status: 'Failed', text: 'Failed' }
	];
	let currentStep: number = 0;
	let isProposer: boolean = false;
	let amountStaked: number;

	let timer: NodeJS.Timer;
	let countdown: string = '^';

	let proposal: ProgramAccount<ProposalType> | undefined = undefined;

	selectedProposal.subscribe(async (p) => {
		proposal = p;

		if (p) {
			currentStep = steps.findIndex((val) => val.text == getVoteStatus(p?.account));
		}

		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk) {
			if (proposal?.account.proposer.toString() == oracleSdk.provider.walletKey.toString()) {
				isProposer = true;
			}
		}

		if (proposal) {
			amountStaked = await (
				await calculateAmountInDecimals(oracleSdk, proposal.account.staked)
			).toNumber();
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

	onDestroy(() => {
		clearInterval(timer);
	});

	async function requestHandler(actionType: string, action: () => Promise<TransactionReceipt>) {
		try {
			const receipt = await action();
			newEvent.set({
				name: `successfully ${actionType}`,
				status: 'success',
				tx: receipt.signature
			});
		} catch (err) {
			const error = err as SendTransactionError;
			newEvent.set({
				name: `failed to ${actionType}`,
				status: 'error',
				message: error.message
			});
		}
	}
	async function collectProposerReward(): Promise<TransactionReceipt> {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && proposal) {
			const collectTx = await oracleSdk
				.proposal()
				.collectProposerRewards({ proposal: proposal.publicKey, tokenMint: SURE_MINT });
			return await collectTx.confirm();
		} else {
			throw new SendTransactionError('could not find valid proposal');
		}
	}

	async function calculateRewards(): Promise<TransactionReceipt> {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && proposal) {
			const finalizeTx = await oracleSdk
				.proposal()
				.finalizeVoteResults({ proposal: proposal.publicKey });
			return await finalizeTx.confirm();
		} else {
			throw new SendTransactionError('could not find valid proposal');
		}
	}
</script>

{#if isProposer}
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
				{#if steps[currentStep].status == 'Failed'}
					<div>
						{#if proposalFailReason(proposal.account) == 'NotEnoughVotes'}
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
				<h3 class=" h3 h3--white ">Available actions</h3>
				{#if steps[currentStep].status == 'Calculate Reward'}
					<CollectRewards title="calculate reward" submitAction={() => calculateRewards()} />
				{:else if steps[currentStep].status == 'Collect Reward'}
					<CollectRewards
						title="collect reward"
						submitAction={() => requestHandler('collect proposer rewards', collectProposerReward)}
					/>
				{:else if steps[currentStep].status == 'Failed'}
					<p>
						{`the ${amountStaked}$sure staked is going into the treasury`}
					</p>
				{:else}
					<p>
						{`there are currently no available actions now`}
					</p>
				{/if}
			{:else}
				<p>Pick a proposal...</p>
			{/if}
		</div>
	</div>
{/if}

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
