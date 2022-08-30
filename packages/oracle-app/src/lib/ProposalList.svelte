<script lang="ts">
	import { css } from '@emotion/css';
	import { BN, type ProgramAccount } from '@project-serum/anchor';
	import type { ProposalType, SureOracleSDK } from '@surec/oracle';
	import { writable } from 'svelte/store';
	import { getProposalStatus } from '@surec/oracle';
	import { onMount } from 'svelte';
	import { walletStore } from '@svelte-on-solana/wallet-adapter-core';
	import { globalStore, createProposalState, selectedProposal } from '../stores/global';
	import {
		prettyPublicKey,
		unixToReadable,
		prettyLargeNumber,
		calculateAmountInDecimals
	} from '$utils';
	import { SURE_MINT_DEV } from './constants';
	const progress = writable(0);
	const proposals = writable<ProgramAccount<ProposalType>[]>([]);

	onMount(async () => {
		const oracleSdk = $globalStore.oracleSDK;
		if (oracleSdk) {
			proposals.set(await oracleSdk.proposal().fetchAllProposals());
			const sureMint = await spl.getMint(oracleSdk.provider.connection, SURE_MINT_DEV);
		}
	});

	walletStore.subscribe(async () => {
		const oracleSdk = $globalStore.oracleSDK;
		const currentTime = new BN(new Date().getUTCSeconds());
		if (oracleSdk) {
			const dd = Math.floor(Date.now() / 1000);
			proposals.set(await oracleSdk.proposal().fetchAllProposals());
		}
	});
</script>

<ul
	class={css`
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 0;
		list-style: none;
		color: white;
		width: 100%;
	`}
>
	{#if $proposals.length > 0}
		{#each $proposals as proposal}
			<li
				on:click={() => selectedProposal.set(proposal)}
				class={css`
					display: flex;
					flex-direction: row;
					background-color: #061e42;
					border-radius: 10px;
					width: 80%;
					margin-bottom: 10px;

					:hover {
						background-color: #082756;
						cursor: pointer;
					}
				`}
			>
				<div
					class={css`
						background-color: #f50093;
						width: 5%;
						border-radius: 10px 0 0 10px;
					`}
				/>
				<div
					class={css`
						padding-top: 2rem;
						padding-bottom: 2rem;
						padding-left: 1rem;
						padding-right: 1rem;
						flex-grow: 2;
					`}
				>
					<div
						class={css`
							display: flex;
							flex-direction: row;
							width: 100%;
							justify-content: space-between;
						`}
					>
						<p class="p p--white p--medium p--margin-0 ">{proposal.account.name}</p>

						<div class={'voting-status'}>
							<p class="p p--small p--pink p--margin-0">
								{`${getProposalStatus(proposal.account)}`}
							</p>
						</div>
					</div>
					<div
						class={css`
							display: flex;
							flex-direction: row;
							justify-content: flex-start;
							gap: 10px;
						`}
					>
						<p class="p p--small p--margin-0">
							{`Proposed: ${unixToReadable(proposal.account.voteStartAt)}`}
						</p>
						<p class="p p--small p--margin-0">
							{`By: ${prettyPublicKey(proposal.account.proposer)}`}
						</p>
						<p class="p p--small p--margin-0">
							{`Staked: ${proposal.account.proposedStaked.toString()}`}
						</p>
					</div>

					<div>
						<p class="p p--medium p--white">{proposal.account.description}</p>
					</div>
					<div>
						<progress
							max={proposal.account.requiredVotes.toNumber()}
							min={0}
							class={css``}
							value={proposal.account.votes}
						/>
						<p class="p p--small p--margin-0">
							{`${prettyLargeNumber(proposal.account.votes)} / ${prettyLargeNumber(
								proposal.account.requiredVotes
							)} required votes`}
						</p>
					</div>
				</div>
			</li>
		{/each}
	{:else}
		<div>
			<p>There is currently no proposals.</p>
			<p>Create a proposal</p>
			<button on:click={() => createProposalState.set(true)}>Create a proposal</button>
		</div>
	{/if}
</ul>

<style lang="scss"></style>
