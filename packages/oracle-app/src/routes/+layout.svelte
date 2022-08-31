<script lang="ts">
	import Header from '$lib/header/Header.svelte';
	import CreateProposal from '$lib/CreateProposal.svelte';
	import { css } from '@emotion/css';
	import { writable } from 'svelte/store';
	import { globalStore } from './../stores/global';
	import { createProposalState } from '../stores/global';
	import TestPanel from '$lib/test/TestPanel.svelte';
	import { onMount } from 'svelte';
	import EventStack from '$lib/EventStack.svelte';

	let showProposal = false;
	onMount(() => {
		createProposalState.subscribe((val) => {
			console.log('create proposal state::');
			showProposal = val;
		});
	});
</script>

<div
	class={css`
		height: 100vh;
	`}
>
	<Header />

	<main class={showProposal ? 'blurred' : ''}>
		<slot />
	</main>
	{#if showProposal}
		<CreateProposal />
	{/if}

	<EventStack />
	{#if process.env.SURE_ENV == 'dev'}
		<TestPanel />
	{/if}
</div>

<style lang="scss">
	@import '../../../sure-static/styles/index.scss';
	:global {
		body {
			background-color: $sure-black;
			width: auto;
			height: 100vh;
		}
	}
	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 1rem;
		width: 100%;
		height: 100vh;
		margin: 0 auto;
		box-sizing: border-box;
		background-color: $sure-black;
	}

	.blurred {
		filter: blur(10px);
	}

	footer {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		padding: 40px;
		background-color: $sure-black;
	}

	footer a {
		font-weight: bold;
	}

	@media (min-width: 480px) {
		footer {
			padding: 40px 0;
		}
	}
</style>
