import { useEffect, useRef } from 'react';
import MainButton from './components/MainButton';
import { css, cx } from '@emotion/css';
import { FieldValues, useForm } from 'react-hook-form';
import { useSureSdk } from './context/sureSdk';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { AnchorProvider } from '@project-serum/anchor';
import { theme } from './components/Themes';
import { prettyPublicKey } from './utils/publickey';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { ErrorMessage } from '@hookform/error-message';
import { useSearchTokenToggle } from './context/searchTokenToggle';
import down from './assets/icons/down.svg';
import SearchTokens from './components/SearchTokens';
import TokenIconInfo from './components/TokenIconInfo';
import GodLife from './assets/icons/godLife.svg';

const ManagePools = () => {
	const sureProgram = useSureSdk();
	const { connection } = useConnection();
	const wallet = useWallet();
	const {
		register,
		handleSubmit,
		formState: { errors },
		setError,
		clearErrors,
	} = useForm();

	const searchTokenToggle = useSearchTokenToggle();
	const manageMarketsRef = useRef(null);

	const onSubmit = handleSubmit(async (data: FieldValues) => {
		if (searchTokenToggle.selectedToken) {
			const programIdPK = new PublicKey(data.programId);
			const tokenMint = new PublicKey(searchTokenToggle.selectedToken?.address);
			if (sureProgram) {
				const txId = await sureProgram?.pool.initializeTokenPool(
					programIdPK,
					tokenMint
				);
			}
		}
	});

	useEffect(() => {
		const listener = sureProgram?.program.addEventListener(
			'CreatePool',
			(event, slot) => {}
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
				<div className="sure-buy-insurance-container">
					<p className="p--margin-s p--large p--white ">Permissionless Pool</p>
					<form
						onSubmit={onSubmit}
						className="action-container-inner-content--form"
					>
						<div className="action-container-inner-content--row">
							<div className="action-container-inner-content--item">
								<p className="p--margin-xs p--small">Pool Name</p>
								<input
									{...register('poolName')}
									className={'input-text-field'}
									type={'text'}
								/>
							</div>
						</div>
						<div
							ref={manageMarketsRef}
							className="action-container-inner-content--row"
						>
							<div className="action-container-inner-content--item">
								<p className="p--margin-xs p--small">ProgramId</p>
								<input
									{...register('programId', { required: 'This is required.' })}
									className={cx(
										'input-text-field__centered',
										`${errors.programId ? 'input-text-field__error' : ''}`
									)}
									onBlur={async (e) => {
										clearErrors('programId');
										try {
											const programIdPk = new PublicKey(e.target.value);
											const account = await connection.getParsedAccountInfo(
												programIdPk
											);
											if (!account.value?.executable) {
												setError('programId', {
													type: 'custom',
													message: 'Invalid ProgramId',
												});
											}
										} catch (err) {
											setError('programId', {
												type: 'custom',
												message: 'Invalid ProgramId',
											});
										}
									}}
									placeholder={prettyPublicKey(PublicKey.default)}
								/>
								<ErrorMessage
									errors={errors}
									name="programId"
									render={({ message }) => (
										<span
											className={cx(
												'p--small',
												css`
													text-align: center;
													background-color: red;
													color: ${theme.colors.sureWhite};
													border-radius: 0px 0px 10px 10px;
													border-width: 1px;
												`
											)}
											role="alert"
										>
											{message}
										</span>
									)}
								/>
							</div>
							<div className="action-container-inner-content--item">
								<p className="p--margin-xs p--small">Token</p>
								<div
									className={css`
										background-color: ${theme.colors.sureBlue4};
										color: ${theme.colors.sureWhite};
										cursor: pointer;
										border-radius: 5px;
										border-width: 1px;
										border-color: transparent;
										padding: 5px;

										display: flex;
										align-items: center;
										justify-content: center;

										&:hover {
											background-color: ${theme.colors.sureBlue2};
										}
									`}
									onClick={() => searchTokenToggle.toggle(true)}
								>
									{searchTokenToggle.selectedToken ? (
										<TokenIconInfo token={searchTokenToggle.selectedToken} />
									) : (
										<>
											<div className={css``}>
												<p className={`p--margin-0 p--small `}>Select Mint</p>
											</div>
											<div className="sure-icon">
												<img src={down} alt="logo" className="icon-small" />
											</div>
										</>
									)}
								</div>
							</div>
						</div>
						{searchTokenToggle.isOpen && (
							<SearchTokens parentRef={manageMarketsRef} />
						)}
						<div className="action-container-inner-content--row_centered">
							{wallet.connected ? (
								<MainButton>
									<h3 className="p--white p--margin-0">Create Pool</h3>
								</MainButton>
							) : (
								<WalletMultiButton />
							)}
						</div>
					</form>
				</div>
			</div>
			<img
				className={css`
					position: absolute;
					right: 0;
					bottom: 0;
					opacity: 30%;
					width: 150px;
					height: 150px;
					z-index: 0;
					:hover {
						opacity: 50%;
					}
				`}
				src={GodLife}
				alt={'god arrow'}
			/>
		</div>
	);
};

export default ManagePools;
