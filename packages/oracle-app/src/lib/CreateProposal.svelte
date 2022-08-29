<script lang="ts">
	import { css, cx } from '@emotion/css';
	import { calculateAccountBalanceFullAmount, calculateAmountInDecimals } from '$utils';
	import close from './../../../sure-static/assets/icons/close.svg';
	import { createProposalState } from './../stores/global';
	import { globalStore, newEvent } from './../stores/global';
	import * as anchor from '@project-serum/anchor';
	import { SURE_MINT_DEV } from './constants';
	import { calculateFullAmount } from '$utils';

	let proposalValues = {
		name: '',
		desription: '',
		stake: 0
	};

	async function submitProposal() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk) {
			try {
				const proposeVoteTx = await oracleSdk.proposal().proposeVote({
					name: proposalValues.name,
					description: proposalValues.desription,
					stake: await calculateFullAmount(oracleSdk, new anchor.BN(proposalValues.stake)),
					mint: SURE_MINT_DEV
				});
				await proposeVoteTx.confirm();
				newEvent.set({
					name: 'successfully create a new proposal'
				});
			} catch (err) {
				console.log('could not propose vote. cause: ', err);
				newEvent.set({
					name: 'could not create a new proposal'
				});
			}
		}
		console.log('values: ', proposalValues);
	}
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<form
	on:submit|preventDefault={submitProposal}
	class={css`
		display: flex;
		justify-content: center;
		background: #102b54;
		position: absolute;
		transform: translateX(-50%);
		top: 20%;
		left: 50%;
		border-radius: 10px;
		width: 30rem;
	`}
>
	<div
		class={css`
			position: absolute;
			color: white;
			right: 10px;
			top: 10px;
			z-index: 2;
		`}
	>
		<img
			src={close}
			class={css`
				//border: black 1px solid;
				border-radius: 100%;
				padding: 1px;
				color: white;
				fill: white;
				:hover {
					cursor: pointer;
					background: #324f7e;
				}
			`}
			width="30"
			on:click={() => createProposalState.set(false)}
			alt="Sure protocol"
		/>
	</div>
	<div
		class={css`
			position: absolute;
			color: white;
			left: 20px;
			top: 0px;
			z-index: 2;
		`}
	>
		<h3 class="h3 p--white">Create proposal</h3>
	</div>
	<div
		class={cx(
			'action-container',
			css`
				background: #102b54;
				width: 10rem;
				padding-left: 2rem;
				padding-right: 2rem;
			`
		)}
	>
		<div class="action-container-inner">
			<div class="action-container-inner-content">
				<p class="p p--white">Name of proposal</p>
				<input
					bind:value={proposalValues.name}
					name="proposalName"
					id="proposalName"
					type="text"
					class="input-text-field"
				/>
			</div>
			<div class="action-container-inner-content">
				<p class="p p--white">Description</p>
				<input
					bind:value={proposalValues.desription}
					name="proposalDescription"
					id="proposalDescription"
					placeholder="an awesome idea"
					type="textarea"
					class="input-text-field"
				/>
			</div>
			<div class="action-container-inner-content">
				<p class="p p--white">Stake</p>
				<input
					bind:value={proposalValues.stake}
					name="proposalStake"
					id="proposalStake"
					type="input"
					class="input-number-field"
				/>
			</div>
			<button class="button">Submit proposal</button>
		</div>
	</div>
</form>

<style lang="scss"></style>
