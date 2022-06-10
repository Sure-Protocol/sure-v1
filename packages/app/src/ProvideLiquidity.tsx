import { css, cx } from '@emotion/css';
import { theme } from './components/Themes';
import { useToggle } from './context/searchToggle';
import { usePool } from './context/surePool';
import down from './assets/icons/down.svg';
import { FieldValues, useForm } from 'react-hook-form';
import MainButton from './components/MainButton';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { useWallet } from '@solana/wallet-adapter-react';
import { useSureSdk } from './context/sureSdk';
import { useEffect } from 'react';
import SearchMarket from './components/SearchMarket';
import { PublicKey } from '@solana/web3.js';
import { useTokens } from './context/tokens';
import { getMint } from '@solana/spl-token';

const ProvideLiquidity: React.FunctionComponent = () => {
	const { register, watch, setValue, getValues, setError, handleSubmit } =
		useForm({});
	const [isOpen, toggle] = useToggle();
	const sureSdk = useSureSdk();
	const [pool] = usePool();
	const wallet = useWallet();
	const tokens = useTokens();

	useEffect(() => {
		register('smartContract');
	}, []);

	useEffect(() => {
		setValue('smartContract', pool?.smartContract);
	}, [pool]);

	useEffect(() => {
		if (pool && sureSdk) {
		}
	}, [watch()]);

	const onSubmit = async (data: FieldValues) => {
		console.log('on sub,it ');
		if (pool && sureSdk) {
			console.log('pool: ', pool);
			const tokenMint = pool.tokenMint;
			console.log('tokenMint: ', pool.tokenMint.toBase58());
			const poolPDA = await sureSdk.pool.getPoolPDA(pool?.smartContract);
			await sureSdk.liquidity.depositLiquidity(
				poolPDA,
				tokenMint,
				data.amount,
				data.rangeStart,
				data.rangeEnd
			);
		}
	};
	return (
		<div className="action-container">
			<div className="action-container-inner">
				<div className="sure-buy-insurance-container">
					<p className="p--margin-s">Provide Liquidity</p>
					<form onSubmit={handleSubmit(onSubmit)} className="">
						<div
							className={css`
								border-radius: 5px;
								margin-right: 1rem;
								flex-grow: 2;
								background-color: ${theme.colors.sureBlue4};
								padding: 4px;
								display: flex;
								flex-direction: row;
								width: fit-content;
							`}
						>
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
								onClick={() => toggle(true)}
							>
								<div className="sure-token--name">
									<p className="p--margin-0 p--white p--bold">{pool?.name}</p>
								</div>
								<div className="sure-icon">
									<img src={down} alt="logo" className="icon-small" />
								</div>
							</div>

							<input
								{...register('amount', { min: 0, valueAsNumber: true })}
								className={'input-number-field'}
								placeholder="0.00"
								typeof="decimals"
							/>
							<button
								{...register('tokenMint')}
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
								{pool && (
									<p className="p--margin-0">
										{tokens?.get(pool?.tokenMint.toBase58())?.symbol}
									</p>
								)}
							</button>
						</div>
						<div
							className={css`
								display: flex;
								flex-wrap: wrap;
								flex-direction: row;
								margin-top: 1rem;
								margin-bottom: 1rem;
								justify-content: center;
							`}
						>
							<div>
								<p className="p--margin-s">Range Start</p>
								<div
									className={css`
										//
										border-radius: 5px;
										margin-right: 1rem;
										flex-grow: 2;
										background-color: ${theme.colors.sureBlue4};
										padding: 4px;

										display: flex;
										flex-direction: row;
										align-items: center;
									`}
								>
									<input
										{...register('rangeStart', {
											min: 0,
											max: 10000,
											valueAsNumber: true,
											onBlur: (e) => {
												if (e.target.value === '') {
													setValue('rangeStart', '10');
												}
												const newValueInt = parseInt(e.target.value);
												const newValue = newValueInt - (newValueInt % 10);

												if (newValue > getValues('rangeEnd')) {
													setValue('rangeStart', getValues('rangeEnd'));
												} else {
													setValue(
														'rangeStart',
														newValue > 0 ? newValue : newValue + 10
													);
												}
											},
											//validate: (value) => value > getValues('rangeEnd'),
										})}
										placeholder="0"
										className={cx(
											'input-number-field',
											css`
												text-align: center;
											`
										)}
									/>
									<p className="p--margin-0 p-margin-center">bp</p>
								</div>
							</div>

							<div>
								<p className="p--margin-s">Range End</p>
								<div
									className={css`
										//
										border-radius: 5px;
										margin-right: 1rem;
										flex-grow: 2;
										background-color: ${theme.colors.sureBlue4};
										padding: 4px;

										display: flex;
										flex-direction: row;
										align-items: center;
									`}
								>
									<input
										{...register('rangeEnd', {
											valueAsNumber: true,
											min: 0,
											max: 10000,
											onBlur: (e) => {
												const newValueInt = parseInt(e.target.value);
												const newValue = newValueInt - (newValueInt % 10);
												setValue('rangeEnd', newValue);
											},
										})}
										placeholder="0"
										className={cx(
											'input-number-field',
											css`
												text-align: center;
											`
										)}
									/>
									<p className="p--margin-0 p-margin-center">bp</p>
								</div>
							</div>
							{isOpen && <SearchMarket />}
						</div>

						<div className="sure-buy-insurance-container--centered">
							<h3 className="h3--white h3--center h3--margin-s">
								Estimated APY: 10.2%
							</h3>
							<p className="p--margin-s p--medium p--center">Pool APY: 10%</p>
							<p className="p--margin-s p--small p--center">
								Premium APY: 0.2%
							</p>
						</div>

						<div className="sure-buy-insurance-container--centered">
							{wallet.connected ? (
								<MainButton>
									<h3 className="p--white p--margin-0">Provide Liquidity</h3>
								</MainButton>
							) : (
								<WalletMultiButton />
							)}
						</div>
					</form>
				</div>
			</div>
		</div>
	);
};

export default ProvideLiquidity;
