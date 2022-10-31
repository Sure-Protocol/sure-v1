<script lang="ts">
	import { prettyLargeNumber, prettyLargeNumberString } from '$lib/utils';
	import { onMount } from 'svelte';
	export let title: string;
	export let amount: string;
	export let loading: boolean;

	const loaderIcons = ['\\', '|', '/', '-'];
	let loaderCounter: number = 0;
	let loader: string = '/';
	let timer: NodeJS.Timer;

	onMount(() => {
		timer = setInterval(() => {
			loader = loaderIcons[loaderCounter % 4];
			loaderCounter += 1;
		}, 200);
	});

	function killTimer(loading: boolean) {
		if (!loading) {
			clearInterval(timer);
		}
	}
	$: o = killTimer(loading);
</script>

<div class={'amount-text'}>
	<p class="p p-small text--margin-vertical__0 amount-text__title">
		{title}
	</p>

	<h1 class="h1 h1--white text--margin-vertical__0 amount-text__amount">
		{#if loading}
			{loader}
		{:else}
			{amount}
		{/if}
	</h1>
</div>

<style lang="scss">
	.amount-text {
		display: flex;
		text-align: center;
		position: relative;
		width: 8rem;

		&__title {
			font-size: 24px;
			bottom: 0px;
			right: 0px;
			transform: translate3d(0%, 20%, 0);
			color: #f50093;
			position: absolute;
			z-index: 0;
		}

		&__amount {
			position: relative;
			z-index: 10;
		}
	}
</style>
