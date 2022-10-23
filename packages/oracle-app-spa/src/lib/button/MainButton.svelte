<script lang="ts">
	import { css, cx } from '@emotion/css';

	export let title: string = '';
	export let inputClass: string = '';
	export let type: string = '';
	export let click: () => void = () => {};
</script>

<button
	on:click={() => click()}
	class={cx(inputClass, 'sure-button', 'hover-illuminated-box')}
	{type}
	aria-disabled="true"
>
	<p class="p p--small p--bold p--white text--margin-vertical__0">{title}</p>
</button>

<style lang="scss">
	.sure-button {
		padding-left: 30px;
		padding-right: 30px;
		padding-top: 2px;
		padding-bottom: 2px;
		height: 2rem;
		border-radius: 10px;
		background-color: #102b54;
		cursor: pointer;
	}
</style>
