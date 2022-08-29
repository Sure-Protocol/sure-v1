<script lang="ts">
	import { css } from '@emotion/css';
	import { newEvent, type Event } from '../stores/global';

	let eventStack: Event[] = [];

	newEvent.subscribe((event) => {
		if (event.name.length > 0) {
			eventStack = [...eventStack, event];
			setTimeout(() => {
				console.log('gon!');
				eventStack = eventStack.slice(0, eventStack.length - 1);
			}, 5000);
			console.log('eventStack: ', eventStack);
		}
	});
</script>

<div
	class={css`
		position: relative;
		bottom: 0;
		right: 0;
	`}
>
	<ul
		class={css`
			list-style: none;
		`}
	>
		{#each eventStack as event}
			<li
				class={css`
					width: 10rem;
					height: 8rem;
					background: blue;
				`}
			>
				{`event: ${event.name}`}
			</li>
		{/each}
		<li />
	</ul>
</div>
