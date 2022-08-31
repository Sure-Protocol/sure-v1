<script lang="ts">
	import { css, cx } from '@emotion/css';

	export let title: string;
	export let description: string;
</script>

<div
	class={css`
		display: flex;
		width: 100%;
		flex-direction: column;
		justify-content: left;
	`}
>
	<p class="p p--white text--margin-vertical__0">{title}</p>
	<p class="p p--small">{description}</p>
	<div
		class={css`
			display: flex;
			justify-content: center;
		`}
	>
		<slot name="input" />
	</div>
</div>
