<script lang="ts">
	import Header from '$lib/header/Header.svelte';
</script>

<Header />

<main>
	<slot />
</main>

<footer />

<style lang="scss">
	@import '../../../sure-static/styles/index.scss';
	:global {
		body {
			background-color: $sure-black;
			width: auto;
		}
	}
	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 1rem;
		width: 100%;
		//max-width: 1024px;
		margin: 0 auto;
		box-sizing: border-box;
		background-color: $sure-black;
	}

	footer {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		padding: 40px;
		background-color: $sure-black;
	}

	footer a {
		font-weight: bold;
	}

	@media (min-width: 480px) {
		footer {
			padding: 40px 0;
		}
	}
</style>
