<script lang="ts">
	export let value: number | undefined;
	export let valueType: string | undefined = undefined;
</script>

<div class="input-sure-amount-box">
	<input bind:value name="proposalStake" id="proposalStake" type="input" class="number-field" />
	{#if valueType}
		<p class="p text--margin-vertical__0">{valueType}</p>
	{/if}
</div>

<style lang="scss">
	@import './../../../../sure-static/styles/index.scss';

	.input-sure-amount-box {
		background-color: #324f7e;
		display: flex;
		flex-direction: row;
		justify-content: center;
		align-items: center;
		border-radius: 5px;
		padding-top: 5px;
		padding-bottom: 5px;
		padding-right: 5px;
		width: 8rem;
	}

	.number-field {
		@extend .input-number-field;
		width: 3rem;
	}
</style>
