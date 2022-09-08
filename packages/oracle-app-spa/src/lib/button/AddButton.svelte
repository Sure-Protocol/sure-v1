<script lang="ts">
	import { css } from '@emotion/css';
	import logo from '$assets/icons/addPlus.svg';

	export let title: string = '';
	export let type: string = '';
	export let click: () => void = () => {};
</script>

<button on:click={() => click()} class={'sure-button'} {type} aria-disabled="true">
	<div
		class={css`
			display: flex;
			flex-direction: row;
			justify-content: center;
			align-items: center;
		`}
	>
		<img src={logo} alt="add proposal" width="25px" />
		<p class="p p--small p--bold p--white text--margin-vertical__0">{title}</p>
	</div>
</button>

<style lang="scss">
	.sure-button {
		padding-left: 10px;
		padding-right: 10px;
		padding-top: 2px;
		padding-bottom: 2px;
		height: 2rem;
		border: grey 1px solid;
		border-radius: 10px;
		background-color: transparent;
		cursor: pointer;

		&:hover {
			border: #f50093 1px solid;
			box-shadow: 0px 0px 6px 3px #f50093;
		}
	}
</style>
