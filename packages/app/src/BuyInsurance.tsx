import MainButton from './components/MainButton';
import InfoBox from './components/InfoBox';
import { SearchProvider, useToggle } from './context/searchToggle';
import { useInsuranceContract } from './context/insuranceContract';
import { usePool } from './context/surePool';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { css } from '@emotion/css';
import SearchMarket from './components/SearchMarket';
import GodArrow from './assets/icons/godArrow.svg';
import { useEffect, useRef, useState } from 'react';
import { FieldValues, useForm } from 'react-hook-form';
import { useSureSdk } from './context/sureSdk';
import WarningBox from './components/WarningBox';
import { SureDate, SureError, SureErrors } from '@surec/sdk';
import MarketSelector from './components/MarketSelector';
import BuyCoverageContent from './components/popup/BuyCoverageContent';
import _ from 'lodash';
import TitleWithPopover from './components/popup/TitleWithPopover';
import BuyCoverageAmountContent from './components/popup/BuyCoverageAmountContent';
import BuyCoverageExpiryContent from './components/popup/BuyCoverageExpiryContent';

const BuyInsurance = () => {
	const { register, watch, setValue, getValues, handleSubmit } = useForm();
	const sureSdk = useSureSdk();
	const [contract] = useInsuranceContract();
	const [pool] = usePool();
	const wallet = useWallet();
	const [isPopoverOpen, setIsPopoverOpen] = useState({
		buyCoverage: false,
		buyCoverageAmount: false,
		buyCoverageExpiry: false,
	});
	const [isOpen, toggle] = useToggle();
	const marketSelectorRef = useRef<HTMLDivElement>(null);

	const [estimate, setEstimate] = useState(['', '', '']);
	const [estimateError, setEstimateError] = useState<{
		errorMsg: string;
		cause: string;
	} | null>(null);

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
					if (err?.error == SureErrors.NotEnoughLiquidity) {
						setEstimateError({
							errorMsg: 'Could not estimate premium',
							cause: SureErrors.NotEnoughLiquidity.name,
						});
					} else {
						setEstimateError({
							errorMsg: 'Could not estimate premium',
							cause: '',
						});
					}
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
		<div
			className={`action-container${
				_.reduce(isPopoverOpen, (res, value, key) => res || value)
					? '__blur'
					: ''
			}`}
		>
			<div className="action-container-inner">
				<div className="action-container-inner-content">
					<TitleWithPopover
						isOpen={_.get(isPopoverOpen, 'buyCoverage')}
						Content={BuyCoverageContent}
						toggle={(open) =>
							setIsPopoverOpen({ ...isPopoverOpen, buyCoverage: open })
						}
					>
						<p className="p--margin-s p--large p--white ">Buy coverage</p>
					</TitleWithPopover>

					<form
						onSubmit={handleSubmit(onSubmit)}
						className="action-container-inner-content--form"
					>
						<div className="action-container-inner-content--row">
							<div className="action-container-inner-content--item">
								<TitleWithPopover
									isOpen={_.get(isPopoverOpen, 'buyCoverageAmount')}
									Content={BuyCoverageAmountContent}
									toggle={(open) =>
										setIsPopoverOpen({
											...isPopoverOpen,
											buyCoverageAmount: open,
										})
									}
								>
									<p className="p--margin-xs p--small">Protocol</p>
								</TitleWithPopover>

								<MarketSelector
									marketRef={marketSelectorRef}
									pool={pool}
									register={register}
								/>
								{pool && (
									<p className="p--small p--margin-s">
										{`Available liquidity in pool ${
											parseInt(pool.liquidity) - parseInt(pool.usedLiquidity)
										} USDC`}
									</p>
								)}
							</div>
							<div className="action-container-inner-content--item">
								<TitleWithPopover
									isOpen={_.get(isPopoverOpen, 'buyCoverageExpiry')}
									Content={BuyCoverageExpiryContent}
									toggle={(open) =>
										setIsPopoverOpen({
											...isPopoverOpen,
											buyCoverageExpiry: open,
										})
									}
								>
									<p className="p--margin-xs p--small">Expiry</p>
								</TitleWithPopover>

								<input
									{...register('expiry')}
									type="date"
									className="sure-buy-insurance-selector--date"
									placeholder={Date.now().toString()}
								/>
							</div>
						</div>
						{isOpen && <SearchMarket parentRef={marketSelectorRef} />}
						{parseInt(contract?.insuredAmount) > 0 && (
							<div className="action-container-inner-content--row__centered">
								<div className="action-container-inner-content--item">
									<p className="p--margin-s p--small">Already covered</p>
									<InfoBox title="Change">
										<div className="sure-buy-insurance-change">
											<div className="sure-buy-insurance-change__status">
												<p className="p--pink">Old</p>
												<p className="p--pink">New</p>
											</div>
											<div className="sure-buy-insurance-change__amount">
												<p className="p">{`${contract.insuredAmount} USDC`}</p>
												<p className="p">{`${getValues('amount')}`}</p>
											</div>
											<div className="sure-buy-insurance-change__date">
												<p className="p">{getValues('expiry')}</p>
												<p className="p"></p>
											</div>
										</div>
									</InfoBox>
								</div>
							</div>
						)}
						{estimate[0] !== '' && (
							<div className="action-container-inner-content--row__centered">
								<p className="p--margin-s p--medium p--center">
									Estimated yearly price
								</p>

								<h3 className="h3--white h3--center h3--margin-s">{`${estimate[1]} USDC`}</h3>
								<p className="p--margin-s p--small p--center">{`Premium ${estimate[2]}bp`}</p>
							</div>
						)}
						{estimateError && (
							<div className="action-container-inner-content--row__centered">
								<div className="action-container-inner-content--item">
									<WarningBox title="Premium">
										<div>
											<p className="p--white p--margin-s p--medium">
												{estimateError.errorMsg}
											</p>
											<p className="p--white p--margin-s p--small">
												{estimateError.cause}
											</p>
										</div>
									</WarningBox>
								</div>
							</div>
						)}

						<div className="action-container-inner-content--row_centered">
							<div className="action-container-inner-content--item">
								{wallet.connected ? (
									<MainButton>
										<h3 className="p--white p--margin-0">Buy</h3>
									</MainButton>
								) : (
									<WalletMultiButton />
								)}
							</div>
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
				src={GodArrow}
				alt={'god arrow'}
			/>
		</div>
	);
};

export default BuyInsurance;
