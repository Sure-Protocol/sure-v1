import type { ProgramAccount } from '@project-serum/anchor';
import * as spl from './../../../../node_modules/@solana/spl-token';
import type { TransactionInstruction } from '@solana/web3';
import type { SureOracleSDK, ConfigType, UpdateConfig } from '@surec/oracle';
import { SURE_MINT, BASE_PK } from '@surec/oracle';
import { writable } from 'svelte/store';
import { newEvent } from './event';
import * as goki from '@gokiprotocol/client';
import { TribecaSDK, getGovernorAddress, GovernorWrapper } from '@tribecahq/tribeca-sdk';
import { getTestKeypairFromSeed } from '$lib/utils';
import type { Update } from 'vite';

export type ConfigState = {
	isLoading: boolean;
	loadingFailed: boolean;
	config: ConfigType | null;
};

export const configState = writable<ConfigState>({
	isLoading: false,
	loadingFailed: false,
	config: null
});

export const loadConfig = () => {
	configState.set({
		isLoading: true,
		loadingFailed: false,
		config: null
	});
};

export const loadedSuccessfully = (config: ConfigType) => {
	configState.set({
		isLoading: true,
		loadingFailed: false,
		config: config
	});
};

export const failedLoading = () => {
	configState.set({
		isLoading: false,
		loadingFailed: true,
		config: null
	});
};

export const hydrateConfig = async (oracleSdk: SureOracleSDK) => {
	loadConfig();
	try {
		const config = await oracleSdk.config().fetchConfig({ tokenMint: SURE_MINT });
		loadedSuccessfully(config);
	} catch (err) {
		failedLoading();
		newEvent.set({
			name: 'failed to load config',
			message: err as string,
			status: 'error'
		});
	}
};

const generatePrettyConfigChange = (
	newConfig: UpdateConfig,
	oldConfig: ConfigState['config']
): string => {
	let prettyUpdate = '';
	if (newConfig.minimumProposalStake) {
		prettyUpdate = `${prettyUpdate}, minimumProposalStake=${newConfig.minimumProposalStake}`;
	}

	if (newConfig.protocolFeeRate) {
		prettyUpdate = `${prettyUpdate}, protocolFeeRate=${newConfig.protocolFeeRate}`;
	}

	if (newConfig.requiredVotes) {
		prettyUpdate = `${prettyUpdate}, requiredVotes=${newConfig.requiredVotes}`;
	}

	if (newConfig.revealPeriod) {
		prettyUpdate = `${prettyUpdate}, revealPeriod=${newConfig.revealPeriod}`;
	}

	if (newConfig.voteStakeRate) {
		prettyUpdate = `${prettyUpdate}, voteStakeRate=${newConfig.voteStakeRate}`;
	}

	if (newConfig.votingPeriod) {
		prettyUpdate = `${prettyUpdate}, votingPeriod=${newConfig.votingPeriod}`;
	}

	return prettyUpdate;
};

export const proposeConfigChange = async ({
	configUpdate,
	config,
	oracleSdk
}: {
	configUpdate: UpdateConfig;
	config: ConfigState['config'];
	oracleSdk: SureOracleSDK;
}) => {
	if (config) {
		const basePk = BASE_PK;

		// tribeca
		const tribecaSdk = TribecaSDK.load({ provider: oracleSdk.provider });
		const governor = getGovernorAddress(basePk);
		const govern = new GovernorWrapper(tribecaSdk, governor);
		const ixs: TransactionInstruction[] = [];
		ixs.push(...(await oracleSdk.config().updateConfigInstructions(configUpdate)));

		const pendingProposal = await govern.createProposal({ instructions: ixs });
		const metadataProposal = await govern.createProposalMeta({
			proposal: pendingProposal.proposal,
			title: 'Update Oracle config parameters',
			descriptionLink: 'https://github.com/orgs/Sure-Protocol/projects'
		});
		const trx = pendingProposal.tx.append(...metadataProposal.instructions);
		await trx.confirm();
	}
};
