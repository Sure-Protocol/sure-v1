import { useContext } from 'react';
import MainButton from './components/MainButton';
import { css } from '@emotion/css';
import { useForm } from 'react-hook-form';
import * as sureSDK from '@sure/sdk';
import { SurePoolProgramContext } from './context/surePool';
import { useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';

interface CreateMarkets {
	protocolName: string;
	ticker: string;
	programId: string;
	token: string;
}

export const ManageMarkets = () => {
	const sureProgram = useContext(SurePoolProgramContext);
	const wallet = useWallet();
	console.log('sureProgram ', sureProgram);
	const {
		register,
		handleSubmit,
		formState: { errors },
	} = useForm();
	const onSubmit = async (data: CreateMarkets) => {
		console.log('Lets go ', data);
		const programIdPK = new PublicKey(data.programId);
		await sureSDK.pool.createPool(
			sureProgram,
			wallet.publicKey,
			programIdPK,
			0
		);
	};

	return (
		<div className="action-container">
			<div className="action-container-inner">
				<form
					className={css`
						display: flex;
						flex-direction: column;
					`}
					onSubmit={handleSubmit(onSubmit)}
				>
					<p>Create new Sure market</p>
					<input {...register('protocolnNme')} placeholder="Protocol Name " />
					<input {...register('ticker')} placeholder="Ticker" />
					<input {...register('programId')} placeholder="Program Id " />
					<input {...register('token')} placeholder="Token" />
					<MainButton>
						<p className="p--white p--margin-0">Submit</p>
					</MainButton>
				</form>
			</div>
		</div>
	);
};
