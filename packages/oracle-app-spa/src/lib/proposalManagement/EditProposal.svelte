<script lang="ts">
	import { css } from '@emotion/css';
	import { onDestroy } from 'svelte';
	import type {
		ProposalType,
		VoteStatus,
		TransactionResult,
	} from '@surec/oracle';
	import { getVoteStatus, SURE_MINT } from '@surec/oracle';
	import { selectedProposal, globalStore, newEvent } from '../../store/index';
	import { SendTransactionError } from '@solana/web3.js';
	import { calculateAmountInDecimals } from '$lib/utils/money.ts';
	import CollectRewards from '$lib/VoteManagement/forms/CollectRewards.svelte';

	let steps: { status: VoteStatus; text: string }[] = [
		{ status: 'Voting', text: 'Voting' },
		{ status: 'Reveal vote', text: 'Reveal vote' },
		{ status: 'Calculate Reward', text: 'Calculate Reward' },
		{ status: 'Collect Reward', text: 'Collect Reward' },
		{ status: 'Failed', text: 'Failed' },
	];
	let currentStep: number = 0;
	let isProposer: boolean = false;
	let amountStaked: number;

	let timer: NodeJS.Timer;
	let countdown: string = '^';

	let proposal: ProposalType | undefined = undefined;

	selectedProposal.subscribe(async (p) => {
		proposal = p;

		if (p) {
			currentStep = steps.findIndex(
				(val) => val.text == getVoteStatus(p?.account)
			);
		}

		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk) {
			if (
				proposal?.account.proposer.toString() ==
				oracleSdk.provider.walletKey.toString()
			) {
				isProposer = true;
			}
		}

		if (proposal) {
			amountStaked = await (
				await calculateAmountInDecimals(oracleSdk, proposal.account.staked)
			).toNumber();
		}
	});

	onDestroy(() => {
		clearInterval(timer);
	});

	async function requestHandler(
		actionType: string,
		action: () => Promise<TransactionResult>
	) {
		try {
			const receipt = await action();
			newEvent.set({
				name: `successfully ${actionType}`,
				status: 'success',
				tx: receipt.signature,
			});
		} catch (err) {
			const error = err as SendTransactionError;
			newEvent.set({
				name: `failed to ${actionType}`,
				status: 'error',
				message: error.message,
			});
		}
	}
	async function collectProposerReward(): Promise<TransactionResult> {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && proposal) {
			const collectTx = await oracleSdk.proposal().collectProposerRewards({
				proposal: proposal.publicKey,
				tokenMint: SURE_MINT,
			});
			return await collectTx.confirm();
		} else {
			throw new SendTransactionError('could not find valid proposal');
		}
	}

	async function calculateRewards(): Promise<TransactionResult> {
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

				<h3 class=" h3 h3--white ">Available actions</h3>
				{#if steps[currentStep].status == 'Calculate Reward'}
					<CollectRewards
						title="calculate reward"
						submitAction={() => calculateRewards()}
					/>
				{:else if steps[currentStep].status == 'Collect Reward'}
					<CollectRewards
						title="collect reward"
						submitAction={() =>
							requestHandler('collect proposer rewards', collectProposerReward)}
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
