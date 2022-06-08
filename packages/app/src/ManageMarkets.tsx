import * as anchor from '@project-serum/anchor';
import { useContext, useEffect } from 'react';
import MainButton from './components/MainButton';
import { css } from '@emotion/css';
import { FieldValue, FieldValues, useForm } from 'react-hook-form';
import { useSureSdk } from './context/sureSdk';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { AnchorProvider } from '@project-serum/anchor';
import { useTokens } from './context/tokens';
import { getAccount } from '@solana/spl-token';

interface CreateMarkets {
	protocolName: string;
	ticker: string;
	programId: string;
	token: string;
}

export const ManageMarkets = () => {
	const sureProgram = useSureSdk();
	const { connection } = useConnection();
	const tokens = useTokens();
	const {
		register,
		handleSubmit,
		formState: { errors },
		setError,
		clearErrors,
	} = useForm();
	console.log('errors: ', errors);

	const onSubmit = handleSubmit(async (data) => {
		const programIdPK = new PublicKey(data.programId);
		const tokenMint = new PublicKey(data.tokenId);
		const poolPDA = await sureProgram?.pool.getOrCreatePool(
			programIdPK,
			10,
			data.programName
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
					<input {...register('programName')} placeholder="Program Name " />
					<input
						{...register('programId')}
						onBlur={async (e) => {
							console.log('e: ', e.target.value);
							clearErrors('programId');
							try {
								const programIdPk = new PublicKey(e.target.value);
								const account = await connection.getParsedAccountInfo(
									programIdPk
								);
								console.log('account: ', account);
								if (!account.value?.executable) {
									setError('programId', {
										type: 'custom',
										message: 'This is not a valid program',
									});
								}
							} catch (err) {
								setError('programId', {
									type: 'custom',
									message: 'Not a valid program',
								});
							}
						}}
						placeholder="Program Id "
					/>
					{errors.programId && (
						<span role="alert">{errors.programId.message}</span>
					)}
					<select
						{...register('tokenId')}
						onBlur={(e) => {
							const tokenId = e.target.value;
							console.log('tokenId: ', tokenId);
						}}
						placeholder="Token program Id "
					>
						{Array.from(tokens?.keys() ?? []).map((token) => (
							<option value={tokens?.get(token)?.address}>
								{tokens?.get(token)?.name}
							</option>
						))}
					</select>
					<MainButton>
						<p className="p--white p--margin-0">Submit</p>
					</MainButton>
				</form>
			</div>
		</div>
	);
};
