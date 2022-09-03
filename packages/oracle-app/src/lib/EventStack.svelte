<script lang="ts">
	import { css, cx } from '@emotion/css';
	import { fade } from 'svelte/transition';
	import { slide } from 'svelte/transition';
	import { quintOut } from 'svelte/easing';

	import { prettyPublicKey } from '$lib/utils';
	import { newEvent, type Event } from '$stores/index';
	import logo from '$assets/icons/openInNew.svg';
	import close from '$assets/icons/close.svg';
	import { prettySolanaExplorerLink } from '$lib/utils/formatting';
	import CloseButton from './button/CloseButton.svelte';
	import { EventEmitter } from '@solana/wallet-adapter-base';

	let eventStack: Event[] = [];

	newEvent.subscribe((event) => {
		if (event.name.length > 0) {
			console.log('new event: ');
			let eventStackTemp = eventStack;
			if (eventStack.length == 4) {
				eventStackTemp = eventStackTemp.slice(1, 4);
			}
			eventStack = [...eventStackTemp, event];

			setTimeout(() => {
				eventStack = eventStack.slice(0, eventStack.length - 1);
			}, 500000);
		}
	});

	function pop(idx: number) {
		eventStack = eventStack.filter((e, i) => i != idx);
	}
</script>

<ul
	id="eventStack"
	class={css`
		display: flex;
		flex-direction: column;
		background-color: transparent;
		position: fixed;
		bottom: 1rem;
		right: 1rem;
		list-style: none;
		margin: 0;
		z-index: 10;
		gap: 1rem;
	`}
>
	{#each eventStack as event, idx}
		<li
			transition:slide={{ delay: 250, duration: 220, easing: quintOut }}
			class={cx('event-item', `event-item__${event.status}`)}
		>
			<p class={cx('status-msg', `status-msg__${event.status}`)}>
				{event.status}
			</p>
			<CloseButton onClick={() => pop(idx)} />
			<h4 class="h4 h4--white text--margin-vertical__0">{`${event.name}`}</h4>
			{#if event.message}
				<p class={'p p--small'}>{event.message}</p>
			{/if}
			{#if event.tx}
				<a
					target="_blank"
					href={prettySolanaExplorerLink(event.tx, 'devnet')}
					class={css`
						display: flex;
						gap: 2px;
					`}
				>
					<p class="p p--small p--white text--margin-vertical__0">
						{`Transaction: ${prettyPublicKey(event.tx)}`}
					</p>
					<img alt="Link to explorer" src={logo} width={'15px'} height={'15px'} />
				</a>
			{/if}
		</li>
	{/each}
	<li />
</ul>

<style lang="scss">
	.event-item {
		display: flex;
		flex-direction: column;
		width: 20rem;
		height: 7rem;
		border: transparent 1px solid;
		border-radius: 10px;
		box-shadow: 1px 1px 1px 1px #09e85d;
		text-align: center;
		align-items: center;
		justify-content: center;
		gap: 1px;

		&__error {
			box-shadow: 1px 1px 1px 1px #d4100b;
		}

		&__info {
			box-shadow: 1px 1px 1px 1px #324f7e;
		}
	}

	.status-msg {
		position: absolute;
		z-index: 10;
		left: 10px;
		transform: translate3d(50%, -4rem, 0);
		text-shadow: #09e85d;
		font-style: italic;
		margin: 0;
		color: #09e85d;
		&__error {
			color: #d4100b;
		}

		&__info {
			color: #324f7e;
		}
	}
</style>
