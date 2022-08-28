<script lang="ts">
	import { css } from '@emotion/css';
	import { globalStore } from '../../stores/global';
	import * as web3 from '@solana/web3.js';
	import * as spl from '@solana/spl-token';
	import type NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';

	async function createCustomMint() {
		const oracleSdk = $globalStore.oracleSDK;

		if (oracleSdk) {
			const mint = await spl.createMint(
				oracleSdk.provider.connection,
				(oracleSdk.provider.wallet as NodeWallet).payer,
				oracleSdk.provider.wallet.publicKey,
				null,
				6
			);
			const mintRes = await spl.mintTo(
				oracleSdk.provider.connection,
				(oracleSdk.provider.wallet as NodeWallet).payer,
				mint,
				oracleSdk.provider.wallet.publicKey,
				oracleSdk.provider.wallet.publicKey,
				100_000_000
			);
		}
	}
	function createSmartWallet() {}
</script>

<div
	class={css`
		position: relative;
		bottom: 0px;
		left: 0px;
		background: gray;

		display: flex;
		justify-content: flex-start;
		gap: 5;
	`}
>
	<button
		class={css`
			background: yellow;
			padding: 10px;
		`}>Mint Sure</button
	>

	<button
		class={css`
			background: yellow;
			padding: 10px;
		`}>Create smart wallet</button
	>

	<button
		class={css`
			background: yellow;
			padding: 10px;
		`}>Create Sure locker</button
	>
</div>

<style lang="scss">
</style>
