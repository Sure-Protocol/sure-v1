<script lang="ts">
	import { css, cx } from '@emotion/css';
	import logo from '$assets/icons/addPlus.svg';

	export let title: string = '';
	export let type: string = '';
	export let click: () => void = () => {};
</script>

<button
	on:click={() => click()}
	class={cx('sure-button', 'hover-illuminated-box')}
	{type}
	aria-disabled="true"
>
	<div
		class={cx(
			css`
				display: flex;
				flex-direction: row;
				justify-content: center;
				align-items: center;
			`
		)}
	>
		<img src={logo} alt="add proposal" width="25px" />
		<p class="p p--small p--bold p--white text--margin-vertical__0">
			{title}
		</p>
	</div>
</button>

<style lang="scss">
	.sure-button {
		margin: 0;
		padding-left: 10px;
		padding-right: 10px;
		padding-top: 2px;
		padding-bottom: 2px;
		height: 2rem;
		border: transparent 2px solid;
		border-radius: 10px;
		background-color: #102b54;
		cursor: pointer;
	}
</style>
