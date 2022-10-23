<script lang="ts">
	import { cx } from '@emotion/css';

	export let value: number | undefined;
	export let inputClass: string | undefined = undefined;
	export let valueType: string | undefined = undefined;
</script>

<div class={cx(inputClass, 'input-sure-amount-box', 'hover-illuminated-box')}>
	<input
		bind:value
		name="proposalStake"
		id="proposalStake"
		type="input"
		class="number-field"
	/>
	{#if valueType}
		<p class="p text--margin-vertical__0">{valueType}</p>
	{/if}
</div>

<style lang="scss">
	@import './../../../../sure-static/styles/index.scss';

	.input-sure-amount-box {
		background-color: #1c3a6c;
		display: flex;
		flex-direction: row;
		justify-content: center;
		align-items: center;
		border-radius: 5px;
		padding-top: 5px;
		padding-bottom: 5px;
		padding-right: 5px;
		width: 8rem;
		border: transparent 2px solid;
	}

	.number-field {
		@extend .input-number-field;
		background-color: #1c3a6c;
		width: 3rem;
	}
</style>
