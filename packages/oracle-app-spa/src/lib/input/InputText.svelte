<script lang="ts">
	import { cx } from '@emotion/css';

	export let value: string | undefined;
	export let validation: () => void = () => {};
	export let valueType: string | undefined = undefined;
	export let textArea: boolean = false;
</script>

<div class={cx('input-sure-text-box', 'hover-illuminated-box')}>
	{#if textArea}
		<textarea
			on:blur={validation}
			class="text-field"
			bind:value
			id="story"
			name="story"
			rows="3"
			cols="33"
		/>
	{:else}
		<input
			on:blur={validation}
			bind:value
			name="proposalStake"
			id="proposalStake"
			type="textarea"
			class="text-field"
		/>
	{/if}
	{#if valueType}
		<p class="p text--margin-vertical__0">{valueType}</p>
	{/if}
</div>

<style lang="scss">
	@import './../../../../sure-static/styles/index.scss';

	.input-sure-text-box {
		background-color: #1c3a6c;
		display: flex;
		flex-direction: row;
		justify-content: center;
		align-items: center;
		border-radius: 5px;
		padding-top: 5px;
		padding-bottom: 5px;
		padding-right: 5px;
		width: 16rem;
	}

	:global(.text-field) {
		@extend .input-text-field;
		background-color: #1c3a6c;
		width: 100%;
	}
</style>
