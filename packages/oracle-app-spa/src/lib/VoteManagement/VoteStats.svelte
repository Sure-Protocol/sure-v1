<script lang="ts">
	import { css } from '@emotion/css';
	import * as anchor from '@project-serum/anchor';
	import { onDestroy } from 'svelte';
	import {
		type ProposalType,
		type VoteStatus,
		type ProposalStatus,
		type VoteAccount,
		SURE_MINT,
	} from '@surec/oracle';
	import {
		getProposalStatus,
		SureOracleSDK,
		getVoteStatus,
	} from '@surec/oracle';
	import { findEscrowAddress } from '@tribecahq/tribeca-sdk';
	import {
		getLockerSdk,
		unixToReadable,
		countdownFromUnix,
		isInFuture,
		getNextDeadline,
		saveSalt,
	} from '$lib/utils';
	import {
		selectedProposal,
		globalStore,
		newEvent,
		tokenState,
	} from '../../store/index';
	import { Steps } from 'svelte-steps';
	import CreateProposal from '../CreateProposal.svelte';
	import UpdateVote from './forms/UpdateVote.svelte';
	import CancelVote from './forms/CancelVote.svelte';
	import RevealVote from './forms/RevealVote.svelte';
	import CollectRewards from './forms/CollectRewards.svelte';
	import type { SendTransactionError } from '@solana/web3.js';
	import StatBox from '$lib/box/StatBox.svelte';
	import { to_number } from 'svelte/internal';
	import errorCircle from '$assets/icons/errorCircle.svg';

	let steps: { status: VoteStatus; text: string }[] = [
		{ status: 'Voting', text: 'Voting' },
		{ status: 'Reveal vote', text: 'Reveal vote' },
		{ status: 'Calculate Reward', text: 'Calculate Reward' },
		{ status: 'Collect Reward', text: 'Collect Reward' },
		{ status: 'Failed', text: 'Failed' },
	];
	let currentStep: number = 0;

	let timer: NodeJS.Timer;
	let countdown: string = '^';
	let vote: VoteAccount | null = null;
	let voteValues = {
		userVote: 0.0,
	};

	let proposal: ProposalType | undefined = undefined;

	selectedProposal.subscribe(async (p) => {
		proposal = p;

		// select vote
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && p) {
			const [votePda] = SureOracleSDK.pda().findVoteAccount({
				proposal: p.publicKey,
				voter: oracleSdk?.provider.wallet.publicKey,
			});
			try {
				const voteAccount = await oracleSdk.program.account.voteAccount.fetch(
					votePda
				);
				vote = voteAccount;
			} catch {
				vote = null;
			}

			// update steps
			currentStep = steps.findIndex(
				(val) => val.text == getVoteStatus(p?.account)
			);
		}
	});

	async function voteOnProposal() {
		const oracleSdk = $globalStore.oracleSDK;
		const lockerSdk = await getLockerSdk(oracleSdk);
		if (oracleSdk && proposal && lockerSdk?.locker) {
			const userVoteQ32 = new anchor.BN(
				Math.floor(voteValues.userVote * Math.pow(2, 32))
			);
			try {
				const [escrowKey] = await tribeca.findEscrowAddress(
					lockerSdk.locker,
					oracleSdk.provider.wallet.publicKey
				);
				const voteTx = await oracleSdk.vote().submitVote({
					vote: userVoteQ32,
					mint: SURE_MINT,
					proposal: proposal.publicKey,
					locker: lockerSdk?.locker,
					userEscrow: escrowKey,
				});
				const txRec = await voteTx.transactionEnvelope.confirm();
				saveSalt(voteTx.salt.toString(), proposal.account.name);

				const [votePda] = SureOracleSDK.pda().findVoteAccount({
					proposal: proposal.publicKey,
					voter: oracleSdk?.provider.wallet.publicKey,
				});
				vote = await oracleSdk.program.account.voteAccount.fetch(votePda);
				newEvent.set({
					name: 'successfully submitted user vote',
					status: 'success',
					tx: txRec.signature,
				});
			} catch (err) {
				const error = err as SendTransactionError;
				newEvent.set({
					name: 'failed to submit vote',
					status: 'error',
					message: error.message,
				});
			}
		}
	}

	async function collectReward() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && proposal) {
			try {
				const [voteAccount] = await oracleSdk.pda.findVoteAccount({
					proposal: proposal.publicKey,
					voter: oracleSdk.provider.wallet.publicKey,
				});
				const voteTx = await oracleSdk.vote().collectRewards({
					voteAccount,
					tokenMint: SURE_MINT,
				});
				const txRec = await voteTx.confirm();
				newEvent.set({
					name: 'successfully collected vote rewards',
					status: 'success',
					tx: txRec.signature,
				});
			} catch (err) {
				const error = err as SendTransactionError;
				newEvent.set({
					name: 'failed to collect vote rewards',
					status: 'error',
					message: error.message,
				});
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
			padding-bottom: 1rem;
		`}
	>
		<h3 class="h3--white">{`Vote management`}</h3>
		{#if proposal !== undefined}
			<p class="p p--italic">{proposal.account.name}</p>

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
						<StatBox title="vote power" value={vote.votePower} />
						<StatBox title="rewards" value={vote.earnedRewards.toNumber()} />
						<StatBox
							title="vote"
							value={vote.revealedVote ? vote.vote.toNumber() : 'NaN'}
						/>
						{#if vote.locked}
							<StatBox title="locked" value={'yes'} />
						{/if}
					</div>
					<h3 class=" h3 h3--white ">Available actions</h3>

					{#if steps[currentStep].status == 'Voting'}
						<div
							class={css`
								display: flex;
								flex-direction: column;
								gap: 1rem;
							`}
						>
							<UpdateVote {proposal} />
							<CancelVote {proposal} />
						</div>
						<div />
					{:else if steps[currentStep].status == 'Reveal vote'}
						<div>
							<RevealVote {proposal} />
						</div>
					{:else if steps[currentStep].status == 'Calculate Reward'}
						<div>Calculate reward</div>
					{:else if steps[currentStep].status == 'Collect Reward'}
						<div>
							<CollectRewards
								title={'Collect rewards'}
								submitAction={() => collectReward()}
							/>
						</div>
					{:else if vote.locked}
						<div>
							<p>
								the vote rewards are collected and the vote is closed. There are
								no more available actions.
							</p>
						</div>
					{:else}
						<div>
							<CollectRewards
								title={'Collect stake'}
								submitAction={() => collectReward()}
							/>
						</div>
					{/if}
				</div>
			{:else if steps[currentStep].status == 'Voting'}
				<div>
					<UpdateVote
						{proposal}
						description={`voting power: ${$tokenState.veSureAmount}`}
						title="Vote"
						buttonTitle={'Vote'}
						submitAction={voteOnProposal}
					/>
				</div>
			{:else}
				<p>no available actions for this vote</p>
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
