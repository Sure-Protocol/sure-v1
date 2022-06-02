import { useContext } from 'react';
import MainButton from './components/MainButton';
import { css } from '@emotion/css';
import { FieldValue, FieldValues, useForm } from 'react-hook-form';
import { useSureSdk } from './context/sureSdk';
import { useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';

interface CreateMarkets {
	protocolName: string;
	ticker: string;
	programId: string;
	token: string;
}

export const ManageMarkets = () => {
	const sureProgram = useSureSdk();
	const wallet = useWallet();
	console.log('sureProgram ', sureProgram);
	const {
		register,
		handleSubmit,
		formState: { errors },
	} = useForm();
	const onSubmit = handleSubmit(async (data) => {
		console.log('Lets go ', data);
		const programIdPK = new PublicKey(data.programId);
		await sureProgram?.pool.createPool(programIdPK, 0);
	});

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
