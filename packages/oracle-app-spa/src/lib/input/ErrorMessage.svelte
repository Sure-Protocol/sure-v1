<script lang="ts">
	import { css } from '@emotion/css';

	export let message: string;
</script>

<div
	class={css`
		padding-top: 10px;
		text-align: left;
	`}
>
	<p class="p text--color__red  text--margin-vertical__0">{message}</p>
</div>
