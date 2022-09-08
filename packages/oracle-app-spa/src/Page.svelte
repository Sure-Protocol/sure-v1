<script lang="ts">
	import TopUp from '$lib/TopUp.svelte';
	import VoteStats from '$lib/VoteManagement/VoteStats.svelte';
	import Proposals from '$lib/proposals/Proposals.svelte';
	import EditProposal from '$lib/proposalManagement/EditProposal.svelte';
	import Config from '$lib/config/Config.svelte';
	import {
		globalStore,
		selectedProposal,
		isOwnerOfProposal,
	} from '$stores/index';
	import ProposalStatus from '$lib/ProposalStatus.svelte';
</script>

<div
	class="action-container action-container--transparent action-container--width-full"
>
	<div class="action-container-inner">
		<div class="action-container-inner-content--row">
			<div class="action-container-inner-content--item">
				<div class="action-container--width-l">
					<Proposals />
				</div>
			</div>
			<div class="action-container-inner-content--item">
				<div class="action-container-inner-content">
					<div class="action-container-inner-content--item">
						<TopUp />
					</div>
					{#if isOwnerOfProposal($selectedProposal?.account, $globalStore?.oracleSDK?.provider.walletKey)}
						<div class="action-container-inner-content--item">
							<ProposalStatus />
						</div>
					{/if}

					<div class="action-container-inner-content--item">
						<EditProposal />
					</div>

					<div class="action-container-inner-content--item">
						<VoteStats />
					</div>
					<div class="action-container-inner-content--item">
						<Config />
					</div>
				</div>
			</div>
		</div>
	</div>
</div>

<style lang="scss" global>
	@import '../../sure-static/styles/index.scss';

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
