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
import { theme } from './components/Themes';

interface CreateMarkets {
	protocolName: string;
	ticker: string;
	programId: string;
	token: string;
}

const ManagePools = () => {
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

	const onSubmit = handleSubmit(async (data) => {
		const programIdPK = new PublicKey(data.programId);
		const tokenMint = new PublicKey(data.tokenId);
		const poolPDA = await sureProgram?.pool.getOrCreatePool(
			programIdPK,
			10,
			data.programName
		);
		await sureProgram?.pool.initializeTokenPool(poolPDA, tokenMint);
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
					<p>Permissionless Pool</p>

					<input
						{...register('programName')}
						className={'input-text-field'}
						placeholder="Program Name "
						type={'text'}
					/>
					<div
						className={css`
							border-radius: 5px;
							margin-right: 1rem;
							flex-grow: 2;
							background-color: ${theme.colors.sureBlue4};
							padding: 4px;
							display: flex;
							flex-direction: row;
						`}
					>
						<input
							{...register('programId')}
							className={'input-text-field'}
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

						<select
							{...register('tokenId')}
							onBlur={(e) => {
								const tokenId = e.target.value;
								console.log('tokenId: ', tokenId);
							}}
							className={css`
								background-color: ${theme.colors.sureBlue4};
								color: ${theme.colors.sureWhite};
								cursor: pointer;
								border-radius: 5px;
								border-width: 1px;
								border-color: transparent;
								padding: 5px;
							`}
						>
							{Array.from(tokens?.keys() ?? []).map((token) => (
								<option value={tokens?.get(token)?.address}>
									{tokens?.get(token)?.name}
								</option>
							))}
						</select>
					</div>
					{errors.programId && (
						<span role="alert">{errors.programId.message}</span>
					)}

					<MainButton>
						<p className="p--white p--margin-0">Submit</p>
					</MainButton>
				</form>
			</div>
		</div>
	);
};

export default ManagePools;
