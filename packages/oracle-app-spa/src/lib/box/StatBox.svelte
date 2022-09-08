<script lang="ts">
	import { cx } from '@emotion/css';

	export let title: string;
	export let value: string | number = '';
</script>

<div class="info-box">
	<p class={cx('p p--small text--margin-vertical__0', 'info-box__title')}>
		{title}
	</p>
	<p class="p p--bold info-box__value">{value}</p>
</div>

<style lang="scss">
	@import './../../../../sure-static/styles/index.scss';
	.info-box {
		position: relative;
		display: flex;
		border: $sure-text-grey 1px solid;
		border-radius: 10px;
		padding-left: 10px;
		padding-right: 10px;
		width: 5rem;
		justify-content: center;
		align-items: center;
		margin-bottom: 1rem;

		&__title {
			position: absolute;
			left: 0px;
			top: 0px;
			transform: translate3d(0%, -60%, 0);
		}

		&__value {
			&:hover {
				color: $sure-pink;
				cursor: none;
			}
		}
	}
</style>
