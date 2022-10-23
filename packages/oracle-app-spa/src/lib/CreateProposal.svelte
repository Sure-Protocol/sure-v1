<script lang="ts">
	import { css, cx } from '@emotion/css';
	import { blur, slide, fly, fade } from 'svelte/transition';
	import { cubicInOut } from 'svelte/easing';
	import {
		createProposalState,
		globalStore,
		hydrateProposalCallback,
		newEvent,
	} from '$stores/index';
	import * as anchor from '@project-serum/anchor';
	import { calculateFullAmount } from '$lib/utils/index';
	import type { SendTransactionError, TransactionError } from '@solana/web3.js';
	import CloseButton from './button/CloseButton.svelte';
	import MainButton from './button/MainButton.svelte';
	import TypeInputAmount from './input/TypeInputAmount.svelte';
	import SingleInput from './input/SingleInput.svelte';
	import InputText from './input/InputText.svelte';
	import { SURE_MINT } from '@surec/oracle';

	let proposalValues = {
		name: '',
		desription: '',
		stake: undefined,
	};

	function validateProposal() {}

	async function submitProposal() {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk && proposalValues.stake) {
			try {
				const proposeVoteTx = await oracleSdk.proposal().proposeVote({
					name: proposalValues.name,
					description: proposalValues.desription,
					stake: await calculateFullAmount(
						oracleSdk,
						new anchor.BN(proposalValues.stake)
					),
					mint: SURE_MINT,
				});
				const txrRes = await proposeVoteTx.confirm();
				newEvent.set({
					name: 'successfully create a new proposal',
					status: 'success',
					tx: txrRes.signature,
				});
				createProposalState.set(false);
			} catch (err) {
				const error = err as SendTransactionError;
				newEvent.set({
					name: 'could not create a new proposal',
					status: 'error',
					message: error.message,
				});
				throw new Error(err);
			}
		}
	}
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<form
	transition:fade={{ delay: 100, duration: 250, easing: cubicInOut }}
	on:submit|preventDefault={async () =>
		hydrateProposalCallback(submitProposal, $globalStore.oracleSDK)}
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
		box-shadow: 10px 10px 40px 1px #324f7e;
	`}
>
	<CloseButton onClick={() => createProposalState.set(false)} />
	<div
		class={css`
			position: absolute;
			color: white;
			left: 20px;
			top: 0px;
			z-index: 2;
		`}
	>
		<h3 class="h3 p--white">_create</h3>
	</div>
	<div
		class={cx(
			'action-container',
			css`
				background: #102b54;
				padding-left: 2rem;
				padding-right: 2rem;
			`
		)}
	>
		<div class="action-container-inner">
			<div
				class={css`
					margin-top: 1rem;
					margin-bottom: 1rem;
				`}
			>
				<SingleInput title="_title" description=".an easy to find title">
					<InputText slot="input" bind:value={proposalValues.name} />
				</SingleInput>
				<SingleInput title="_about" description="what will be decided">
					<InputText
						slot="input"
						bind:value={proposalValues.desription}
						textArea
					/>
				</SingleInput>
				<SingleInput
					title="_stake"
					description="the amount of $sure you are willing to stake on being correct"
				>
					<TypeInputAmount
						inputClass={'create-proposal-input'}
						slot="input"
						bind:value={proposalValues.stake}
						valueType="$sure"
					/>
				</SingleInput>
			</div>

			<MainButton
				inputClass={css`
					border: #324f7e 2px solid;
				`}
				title="Submit"
				type="submit"
			/>
		</div>
	</div>
</form>

<style lang="scss">
	:global(.create-proposal-input) {
		border: #9ca3af 2px solid;
	}
</style>
