<script lang="ts">
	import { css, cx } from '@emotion/css';

	export let inputClass: string | undefined = '';
	export let title: string;
	export let description: string | undefined = undefined;
</script>

<div
	class={cx(
		css`
			display: flex;
			width: 100%;
			flex-direction: column;
			justify-content: left;
		`,
		inputClass
	)}
>
	<p class="p p--white text--margin-vertical__0">{title}</p>
	{#if description}
		<p class="p p--small text--margin-vertical__s">{description}</p>
	{/if}
	<div
		class={css`
			display: flex;
			justify-content: center;
		`}
	>
		<slot name="input" />
	</div>
</div>
