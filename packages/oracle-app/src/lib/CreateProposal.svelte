<script lang="ts">
	import { css, cx } from '@emotion/css';
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<div
	class={css`
		background: blue;
		position: absolute;
		transform: translateX(-50%);
		width: 10rem;
		height: 10rem;
		top: 20%;
		left: 50%;
		border-radius: 10px;
	`}
>
	<form
		class={css`
			background-color: #324f7e;
		`}
	>
		<div
			class={cx(
				'action-container',
				css`
					background: #324f7e;
					width: 10rem;
				`
			)}
		>
			<div class="action-container-inner">
				<div class="action-container-inner-content">
					<p>Name of proposal</p>
					<input type="text" class="input-text-field" />
				</div>
			</div>
		</div>
	</form>
</div>

<style lang="scss"></style>
