<script lang="ts">
	import { css } from '@emotion/css';
	import { globalStore, newEvent } from '$stores/index';
	import type { ProgramAccount } from '@project-serum/anchor';
	import MainButton from '$lib/button/MainButton.svelte';
	import type { SendTransactionError } from '@solana/web3.js';

	type action<T> = () => Promise<T>;
	export let title: string;
	export let submitAction: action<void>;
</script>

<form
	class={css`
		display: flex;
		flex-direction: row;
		gap: 10px;
	`}
	on:submit|preventDefault={async () => submitAction()}
>
	<div
		class={css`
			display: flex;
			flex-direction: column;
		`}
	>
		<MainButton {title} type={'submit'} />
	</div>
</form>

<style lang="scss">
</style>
