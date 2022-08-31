<script lang="ts">
	import { css } from '@emotion/css';
	import close from '$assets/icons/close.svg';

	export let onClick: () => void = () => {};
</script>

<div
	class={css`
		position: absolute;
		color: white;
		right: 10px;
		z-index: 2;
		transform: translate3d(0%, 0%, 0);
	`}
>
	<img
		src={close}
		class={css`
			//border: black 1px solid;
			border-radius: 100%;
			padding: 1px;
			color: white;
			fill: white;
			:hover {
				cursor: pointer;
				background: #324f7e;
			}
		`}
		width="30"
		on:click={() => onClick()}
		alt="Sure protocol"
	/>
</div>
