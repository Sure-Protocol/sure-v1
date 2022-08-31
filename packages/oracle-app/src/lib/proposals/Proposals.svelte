<script lang="ts">
	import { css, cx } from '@emotion/css';
	import ProposalList from '$lib/proposals/ProposalList.svelte';
	import MainButton from '$lib/button/MainButton.svelte';
	import AddButton from '$lib/button/AddButton.svelte';
	import { createProposalState } from '$stores/global';

	let search: string = '';
</script>

<div
	class={cx(
		'action-container-inner',
		css`
			display: flex;
			flex-direction: column;
			justify-content: flex-start;
			gap: 2rem;
			padding: 0;
			height: 100vh;
		`
	)}
>
	<h2 class="h3--white text--margin-vertical__0">Proposals</h2>
	<p class="p text--margin-vertical__0">Query or create a new proposal</p>
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
				border-radius: 5px;
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
					css`
						width: 10vw;
						padding-left: 30px;
						position: relative;
						border: transparent 1px solid;
						box-shadow: 0px 0px 10px 1px #f50093;

						:focus {
							width: 30vw;
							border: #f50093 1px solid;
							box-shadow: 0px 0px 6px 3px #f50093;
						}
					`
				)}
				placeholder="search proposals"
			/>
		</div>
		<AddButton title="create proposal" click={() => createProposalState.set(true)} />
	</div>
	<ProposalList {search} />
</div>
