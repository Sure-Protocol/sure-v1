<script lang="ts">
	import { css, cx } from '@emotion/css';
	import { blur, slide, fly, fade } from 'svelte/transition';
	import { cubicInOut } from 'svelte/easing';
	import {
		createProposalState,
		globalStore,
		hydrateProposalCallback,
		newEvent,
		tokenState,
	} from '$stores/index';
	import * as anchor from '@project-serum/anchor';
	import { calculateFullAmount } from '$lib/utils/index';
	import type { SendTransactionError } from '@solana/web3.js';
	import CloseButton from './button/CloseButton.svelte';
	import MainButton from './button/MainButton.svelte';
	import TypeInputAmount from './input/TypeInputAmount.svelte';
	import SingleInput from './input/SingleInput.svelte';
	import InputText from './input/InputText.svelte';
	import { SURE_MINT } from '@surec/oracle';
	import ErrorMessage from './input/ErrorMessage.svelte';

	let proposalValues = {
		name: '',
		description: '',
		stake: undefined,
	};

	let errorValues = {
		name: '',
		description: '',
		stake: '',
		default: '',
	};

	function validateProposal(valueName: string) {
		return () => {
			switch (valueName) {
				case 'name':
					errorValues.name = '';
					if (proposalValues.name.length < 3)
						errorValues.name = 'Your grand idea needs a name';
					break;
				case 'description':
					errorValues.description = '';
					if (proposalValues.description.length < 10)
						errorValues.description = 'Why not describe your idea';
					break;
				case 'stake':
					errorValues.stake = '';
					if (!proposalValues.stake || proposalValues.stake < 10)
						errorValues.stake = 'Please put your money where your mouth is';
					break;
			}
		};
	}

	function anyErrors(errorValues): boolean {
		return Object.keys(errorValues).some((v) => errorValues[v].length > 0);
	}

	async function submitForm() {
		try {
			await hydrateProposalCallback(submitProposal, $globalStore.oracleSDK);
		} catch (err) {
			const cleanError = `${err}`.replaceAll('Error:', '');
			errorValues.default = `Could not create proposal. Cause: ${
				cleanError.charAt(0).toUpperCase() + cleanError.slice(1)
			}`;
		}
	}

	async function submitProposal() {
		validateProposal('name')();
		validateProposal('description')();
		validateProposal('stake')();
		// only try submit if validation is cleared
		if (!anyErrors(errorValues)) {
			const oracleSdk = $globalStore.oracleSDK;
			if (oracleSdk && proposalValues.stake) {
				try {
					const proposeVoteTx = await oracleSdk.proposal().proposeVote({
						name: proposalValues.name,
						description: proposalValues.description,
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
		} else {
			errorValues.default =
				'Could not submit the proposal since some fields are missing.';
		}
	}
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<form
	transition:fade={{ delay: 100, duration: 250, easing: cubicInOut }}
	on:submit|preventDefault={submitForm}
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
					<div slot="input">
						<InputText
							bind:value={proposalValues.name}
							validation={validateProposal('name')}
						/>
						<ErrorMessage message={errorValues.name} />
					</div>
				</SingleInput>
				<SingleInput title="_about" description="what will be decided">
					<div slot="input">
						<InputText
							bind:value={proposalValues.description}
							validation={validateProposal('description')}
							textArea
						/>
						<ErrorMessage message={errorValues.description} />
					</div>
				</SingleInput>
				<SingleInput
					title="_stake"
					description="the amount of $sure you are willing to stake on being correct"
				>
					<div slot="input">
						<TypeInputAmount
							inputClass={'create-proposal-input'}
							maxValue={$tokenState.sureAmount}
							bind:value={proposalValues.stake}
							validation={validateProposal('stake')}
							valueType="$sure"
						/>
						<ErrorMessage message={errorValues.stake} />
					</div>
				</SingleInput>
			</div>

			<MainButton
				inputClass={css`
					border: #324f7e 2px solid;
				`}
				title="Submit"
				type="submit"
			/>
			<ErrorMessage message={errorValues.default} />
		</div>
	</div>
</form>

<style lang="scss">
	:global(.create-proposal-input) {
		border: #9ca3af 2px solid;
	}
</style>
