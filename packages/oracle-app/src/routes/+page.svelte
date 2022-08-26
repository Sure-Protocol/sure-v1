<script lang="ts">
	import Counter from '$lib/Counter.svelte';
	import { css } from '@emotion/css';
	import { walletStore } from '@svelte-on-solana/wallet-adapter-core';
	import { writable } from 'svelte/store';
	import * as sureOracle from '../../../oracle-sdk/src/index';

	const progress = writable(0);
	progress.set(0.3);
	$: console.log('walletStore: ', walletStore);
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<div class="action-container action-container--transparent action-container--width-full">
	<div class="action-container-inner">
		<div class="action-container-inner-content--row">
			<div class="action-container-inner-content--item">
				<div class="action-container--width-l">
					<div class="action-container-inner">
						<h2 class="h3--white">Votes</h2>
						<div>
							<input type="text" class="input-text-field" placeholder="search votes" />
							<button>Filter</button>
						</div>

						<ul
							class={css`
								display: flex;
								flex-direction: column;
								align-items: center;
								padding: 0;
								list-style: none;
								color: white;
								width: 100%;
							`}
						>
							<li
								class={css`
									display: flex;
									flex-direction: row;
									background-color: #061e42;
									border-radius: 10px;
									width: 80%;
									margin-bottom: 10px;

									:hover {
										background-color: #082756;
										cursor: pointer;
									}
								`}
							>
								<div
									class={css`
										background-color: #f50093;
										width: 5%;
										border-radius: 10px 0 0 10px;
									`}
								/>
								<div
									class={css`
										padding-top: 2rem;
										padding-bottom: 2rem;
										padding-left: 1rem;
										padding-right: 1rem;
										flex-grow: 2;
									`}
								>
									<div
										class={css`
											display: flex;
											flex-direction: row;
											width: 100%;
											justify-content: space-between;
										`}
									>
										<p class="p p--white p--medium p--margin-0 ">Lost money on x</p>

										<div class={'voting-status'}>
											<p class="p p--small p--pink p--margin-0">Voting</p>
										</div>
									</div>
									<div
										class={css`
											display: flex;
											flex-direction: row;
											justify-content: flex-start;
											gap: 10px;
										`}
									>
										<p class="p p--small p--margin-0">Proposed: 10/10/10</p>
										<p class="p p--small p--margin-0">By: h332j32...232e</p>
									</div>
									<div>
										<p class="p p--medium p--white">this is lost of money caused</p>
									</div>
									<div>
										<progress class={css``} value={$progress} />
										<p class="p p--small p--margin-0">200000 / 1_000_000 required votes</p>
									</div>
								</div>
							</li>
						</ul>
					</div>
				</div>
			</div>
			<div class="action-container-inner-content--item">
				<div class="action-container-inner-content">
					<div class="action-container-inner-content--item">
						<div class="action-container--width-s action-container--padding-h0 ">
							<div
								class={css`
									width: 100%;
									color: white;
								`}
							>
								<h3 class="h3--white">Top up with veSure</h3>
								<input type="number" class="input-number-field--padding-m" />
								<p>Amount of sure: 0</p>
								<p>Amount of veSure: 0</p>
								<button>Lock</button>
								<button>Unlock</button>
							</div>
						</div>
					</div>
					<div class="action-container-inner-content--item">
						<div class="action-container--width-s action-container--padding-h0 ">
							<div
								class={css`
									width: 100%;
									color: white;
								`}
							>
								<h3 class="h3--white">Vote stats</h3>
								<p>Pick a proposal...</p>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>

<style lang="scss" global>
	@import '../../../sure-static/styles/index.scss';

	progress {
		border-radius: 0px;
		width: 80%;
		height: 10px;
		box-shadow: 1px 1px 4px rgba(0, 0, 0, 0.2);
	}
	progress::-webkit-progress-bar {
		background-color: white;
		border-radius: 0px;
	}
	progress::-webkit-progress-value {
		background-color: $sure-pink;
		//border-radius: 7px;
		box-shadow: 1px 1px 1px 1px rgba(0, 0, 0, 0.8);
	}
	progress::-moz-progress-bar {
		/* style rules */
	}

	.voting-status {
		border: $sure-pink 1px solid;
		border-radius: 10px;
		padding-left: 10px;
		padding-right: 10px;
		padding-top: 5px;
		padding-bottom: 5px;
	}

	section {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		flex: 1;
	}

	h1 {
		width: 100%;
	}
</style>
