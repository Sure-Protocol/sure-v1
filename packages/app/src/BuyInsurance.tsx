import * as anchor from '@project-serum/anchor';
import MainButton from './components/MainButton';
import InfoBox from './components/InfoBox';
import DateSelector from './components/DateSelector';
import { SearchProvider, useToggle } from './context/searchToggle';
import { useInsuranceContract } from './context/insuranceContract';
import { usePool } from './context/surePool';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { css } from '@emotion/css';
import { theme } from './components/Themes';
import SearchMarket from './components/SearchMarket';
import down from './assets/icons/down.svg';
import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useSureSdk } from './context/sureSdk';
import { PublicKey } from '@solana/web3.js';
import WarningBox from './components/WarningBox';

const BuyInsurance = () => {
	const { register, watch, setValue, getValues, handleSubmit } = useForm();
	const sureSdk = useSureSdk();
	const [contract] = useInsuranceContract();
	const [pool] = usePool();
	const wallet = useWallet();
	const [isOpen, toggle] = useToggle();

	const [estimate, setEstimate] = useState([
		new anchor.BN(0),
		new anchor.BN(0),
	]);
	const [estimateError, setEstimateError] = useState('');

	useEffect(() => {
		register('smartContract');
	}, []);

	useEffect(() => {
		setValue('smartContract', pool?.smartContract);
	}, [pool]);

	useEffect(() => {
		if (pool && sureSdk) {
			const estimateYearlyPremium = async () => {
				const poolPDA = await sureSdk.insurance.getPoolPDA(pool.smartContract);
				const amount = getValues('amount');
				const tokenMint = new PublicKey(
					'GtRBUokeS2cZGTExX2LtEkpqQrU3P9vQ1pVJ7sSmf5N5'
				);
				try {
					const estimate = await sureSdk?.insurance.estimateYearlyPremium(
						amount,
						tokenMint,
						poolPDA
					);
					if (estimate) {
						setEstimate([estimate[0], estimate[1]]);
					}
				} catch (err) {
					setEstimateError('Could not estimate premium');
				}
			};
			estimateYearlyPremium();
		}
	}, [watch()]);

	const onSubmit = async (data) => {};

	return (
		<div className="action-container">
			<div className="action-container-inner">
				<div className="sure-buy-insurance-container">
					<p className="p--margin-s">Buy Coverage</p>
					<form
						onSubmit={handleSubmit(onSubmit)}
						className="sure-buy-insurance-selectors--horisontal"
					>
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
								<div className="sure-token">{pool?.name}</div>
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
								<p className="p--margin-0">{'USDC'}</p>
							</button>
						</div>

						<div className="sure-buy-insurance-selector--date">
							<input
								{...register('expiry')}
								type="date"
								className={css`
									background-color: transparent;
									border-radius: 5px;
									border-width: 1px;
									border-color: transparent;
									padding: 5px;
									width: fit-content;
									text-align: center;
									color: ${theme.colors.sureWhite};
									&:focus {
										outline: none;
									}
								`}
								placeholder="10.August 2022"
							/>
						</div>
					</form>
					{isOpen && <SearchMarket />}

					{pool && (
						<p className="p--small p--margin-s">
							{`Available liquidity in pool ${pool.liquidity} USDC`}
						</p>
					)}
				</div>
				{contract?.insuredAmount.gten(0) && (
					<div className="sure-buy-insurance-container">
						<p className="p--margin-s p--small">Already covered</p>
						<InfoBox title="Change">
							<div className="sure-buy-insurance-change">
								<div className="sure-buy-insurance-change__status">
									<p className="p--pink">Old</p>
									<p className="p--pink">New</p>
								</div>
								<div className="sure-buy-insurance-change__amount">
									<p className="p">10,000000 USDC</p>
									<p className="p">10,000 USDC</p>
								</div>
								<div className="sure-buy-insurance-change__date">
									<p className="p">1. June 2022</p>
									<p className="p">28. August 2022</p>
								</div>
							</div>
						</InfoBox>
					</div>
				)}
				{estimate[0].gtn(0) && (
					<div className="sure-buy-insurance-container--centered">
						<p className="p--margin-s p--medium p--center">Estimated price</p>

						<h3 className="h3--white h3--center h3--margin-s">{`${estimate[0]} USDC`}</h3>
						<p className="p--margin-s p--small p--center">Premium: 2.4%</p>
					</div>
				)}
				{estimateError && (
					<WarningBox title="Premium">
						<p className="h3--white h3--margin-s">Could not estimate premium</p>
					</WarningBox>
				)}

				<div className="sure-buy-insurance-container--centered">
					{wallet.connected ? (
						<MainButton>
							<h3 className="p--white p--margin-0">Buy</h3>
						</MainButton>
					) : (
						<WalletMultiButton>
							<h3 className="p--white p--margin-0">Connect to buy</h3>
						</WalletMultiButton>
					)}
				</div>
			</div>
		</div>
	);
};

export default BuyInsurance;
