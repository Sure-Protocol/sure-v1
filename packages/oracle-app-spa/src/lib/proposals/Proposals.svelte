<script lang="ts">
	import { css, cx } from '@emotion/css';
	import ProposalList from '$lib/proposals/ProposalList.svelte';
	import MainButton from '$lib/button/MainButton.svelte';
	import AddButton from '$lib/button/AddButton.svelte';
	import { createProposalState } from '../../store/index';

	let search: string = '';
</script>

<div
	class={cx(
		'action-container-inner',
		css`
			display: flex;
			flex-direction: column;
			justify-content: flex-start;
			gap: 5rem;
			padding: 0;
			height: 100vh;
		`
	)}
>
	<div
		class={css`
			width: 70%;
			display: flex;
			flex-direction: column;
			gap: 1rem;
		`}
	>
		<h2 class="h3--white text--margin-vertical__0">
			Decentralized prediction market
		</h2>
		<p class="p text--margin-vertical__0">
			.get rewarded for predicting the future or agreeing on off-chain data.
		</p>
		<p class="p text--margin-vertical__0">
			.propose questions you want to have answered.
		</p>
	</div>

	<div
		class={css`
			display: flex;
			gap: 1rem;
		`}
	>
		<div
			class={css`
				position: relative;
				display: flex;
				align-items: center;
				width: 20vw;
				background-color: #324f7e;
				border: transparent 1px solid;
				border-radius: 8px;
			`}
		>
			<p
				class={cx(
					'p text--margin-vertical__0',
					css`
						position: absolute;
						z-index: 5;
						left: 10px;
						color: #f50093;
					`
				)}
			>
				>
			</p>
			<input
				bind:value={search}
				type="text"
				class={cx(
					'input-text-field',
					'hover-illuminated-box',
					css`
						margin-bottom: 0px;
						height: 100%;
						width: 10vw;
						padding-left: 30px;
						position: relative;
					`
				)}
				placeholder="search proposals"
			/>
		</div>
		<AddButton title="new" click={() => createProposalState.set(true)} />
	</div>
	<ProposalList {search} />
</div>

<style lang="scss">
</style>
