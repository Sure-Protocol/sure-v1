import * as anchor from '@project-serum/anchor';
import { useContext, useEffect } from 'react';
import MainButton from './components/MainButton';
import { css } from '@emotion/css';
import { FieldValue, FieldValues, useForm } from 'react-hook-form';
import { useSureSdk } from './context/sureSdk';
import { useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { AnchorProvider } from '@project-serum/anchor';

interface CreateMarkets {
	protocolName: string;
	ticker: string;
	programId: string;
	token: string;
}

export const ManageMarkets = () => {
	const sureProgram = useSureSdk();
	const {
		register,
		handleSubmit,
		formState: { errors },
	} = useForm();

	const onSubmit = handleSubmit(async (data) => {
		console.log('Lets go ', data);
		const programIdPK = new PublicKey(data.programId);
		const tokenMint = new PublicKey(data.tokenId);
		const poolPDA = await sureProgram?.pool.getOrCreatePool(
			programIdPK,
			10,
			data.protocolnName
		);
		await sureProgram?.pool.createPoolVault(tokenMint, programIdPK);
	});

	useEffect(() => {
		const listener = sureProgram?.program.addEventListener(
			'CreatePool',
			(event, slot) => {
				console.log('Event: CreatePoool');
				console.log('event: ', event);
			}
		);

		async () => {
			if (listener) {
				await sureProgram?.program.removeEventListener(listener);
			}
		};
	}, []);

	return (
		<div className="action-container">
			<div className="action-container-inner">
				<form
					className={css`
						display: flex;
						flex-direction: column;
					`}
					onSubmit={onSubmit}
				>
					<p>Create new Sure market</p>
					<input {...register('protocolnName')} placeholder="Protocol Name " />
					<input {...register('ticker')} placeholder="Ticker" />
					<input {...register('programId')} placeholder="Program Id " />
					<input {...register('tokenId')} placeholder="Token program Id " />
					<MainButton>
						<p className="p--white p--margin-0">Submit</p>
					</MainButton>
				</form>
			</div>
		</div>
	);
};
