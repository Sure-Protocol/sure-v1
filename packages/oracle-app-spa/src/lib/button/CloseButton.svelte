<script lang="ts">
	import { css, cx } from '@emotion/css';
	import close from '$assets/icons/close.svg';

	export let onClick: () => void = () => {};
</script>

<div
	class={cx(
		css`
			position: absolute;
			color: white;
			right: 10px;
			z-index: 2;
			transform: translate3d(0%, 0%, 0);
		`
	)}
>
	<img
		src={close}
		class={css`
			//border: black 1px solid;
			border-radius: 100%;
			padding: 1px;
			color: white;
			fill: white;
			border: transparent 2px solid;
			:hover {
				fill: #f50093;
				cursor: pointer;
				transition: border 200ms, box-shadow 400ms;
				border: #f50093 2px solid;
				border-radius: 100%;
				box-shadow: 0px 0px 30px 1px #f50093;
			}
		`}
		width="30"
		on:click={() => onClick()}
		alt="Sure protocol"
	/>
</div>
