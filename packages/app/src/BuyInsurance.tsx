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
import { useEffect, useRef, useState } from 'react';
import { FieldValues, useForm } from 'react-hook-form';
import { useSureSdk } from './context/sureSdk';
import { PublicKey } from '@solana/web3.js';
import WarningBox from './components/WarningBox';
import { SureDate } from '@surec/sdk';
import MarketSelector from './components/MarketSelector';

const BuyInsurance = () => {
	const { register, watch, setValue, getValues, handleSubmit } = useForm();
	const sureSdk = useSureSdk();
	const [contract] = useInsuranceContract();
	const [pool] = usePool();
	const wallet = useWallet();
	const [isOpen, toggle] = useToggle();
	const marketSelectorRef = useRef<HTMLDivElement>(null);

	const [estimate, setEstimate] = useState(['', '', '']);
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
				const tokenMint = pool.tokenMint;
				try {
					const estimate = await sureSdk?.insurance.estimateYearlyPremium(
						amount,
						tokenMint,
						poolPDA
					);
					if (estimate) {
						setEstimate([estimate[0], estimate[1], estimate[2]]);
					}
				} catch (err) {
					setEstimateError('Could not estimate premium');
				}
			};
			estimateYearlyPremium();
		}
	}, [watch('amount')]);

	const onSubmit = async (data: FieldValues) => {
		if (sureSdk && pool) {
			const expiryDateInMs = Date.parse(data.expiry);
			const expiryDateInS = SureDate.new(expiryDateInMs).getTimeInSeconds();
			const poolPDA = await sureSdk.insurance.getPoolPDA(pool.smartContract);
			await sureSdk?.insurance.buyInsurance(
				poolPDA,
				pool.tokenMint,
				data.amount,
				expiryDateInS
			);
		}
	};

	return (
		<div className="action-container">
			<div className="action-container-inner">
				<div className="action-container-inner-content">
					<p className="p--margin-s p--large p--white ">Buy coverage</p>
					<form
						onSubmit={handleSubmit(onSubmit)}
						className="action-container-inner-content--form"
					>
						<div className="action-container-inner-content--row">
							<div className="action-container-inner-content--item">
								<p className="p--margin-xs p--small">Amount</p>
								<MarketSelector
									marketRef={marketSelectorRef}
									pool={pool}
									register={register}
								/>
							</div>
							<div className="action-container-inner-content--item">
								<p className="p--margin-xs p--small">Expiry</p>
								<input
									{...register('expiry')}
									type="date"
									className="sure-buy-insurance-selector--date"
									placeholder="10.August 2022"
								/>
							</div>
						</div>
						{isOpen && <SearchMarket parentRef={marketSelectorRef} />}
					</form>

					{pool && (
						<p className="p--small p--margin-s">
							{`Available liquidity in pool ${pool.liquidity} USDC`}
						</p>
					)}
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
										<p className="p">{`${contract.insuredAmount} USDC`}</p>
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
					{estimate[0] !== '' && (
						<div className="sure-buy-insurance-container--centered">
							<p className="p--margin-s p--medium p--center">
								Estimated yearly price
							</p>

							<h3 className="h3--white h3--center h3--margin-s">{`${estimate[1]} USDC`}</h3>
							<p className="p--margin-s p--small p--center">{`Premium ${estimate[2]}bp`}</p>
						</div>
					)}
					{estimateError && (
						<WarningBox title="Premium">
							<p className="h3--white h3--margin-s">
								Could not estimate premium
							</p>
						</WarningBox>
					)}

					<div className="action-container-inner-content--row_centered">
						{wallet.connected ? (
							<MainButton>
								<h3 className="p--white p--margin-0">Buy</h3>
							</MainButton>
						) : (
							<WalletMultiButton />
						)}
					</div>
				</div>
			</div>
		</div>
	);
};

export default BuyInsurance;
