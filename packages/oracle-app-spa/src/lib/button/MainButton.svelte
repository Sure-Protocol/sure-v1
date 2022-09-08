<script lang="ts">
	import { css } from '@emotion/css';

	export let title: string = '';
	export let type: string = '';
	export let click: () => void = () => {};
</script>

<button on:click={() => click()} class={'sure-button'} {type} aria-disabled="true">
	<p class="p p--small p--bold p--white text--margin-vertical__0">{title}</p>
</button>

<style lang="scss">
	.sure-button {
		padding-left: 30px;
		padding-right: 30px;
		padding-top: 2px;
		padding-bottom: 2px;
		height: 2rem;
		border: transparent 1px solid;
		box-shadow: 0px 0px 10px 1px #f50093;
		border-radius: 10px;
		background-color: transparent;
		cursor: pointer;

		&:hover {
			border: #f50093 1px solid;
			box-shadow: 0px 0px 6px 3px #f50093;
		}
	}
</style>
