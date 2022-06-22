import { css } from '@emotion/css';
import { useToggle } from './context/searchToggle';
import { usePool } from './context/surePool';
import { FieldValues, useForm } from 'react-hook-form';
import MainButton from './components/MainButton';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { useWallet } from '@solana/wallet-adapter-react';
import { useSureSdk } from './context/sureSdk';
import { useEffect, useReducer, useRef, useState } from 'react';
import SearchMarket from './components/SearchMarket';
import MarketSelector from './components/MarketSelector';
import NumberUnitInputSelector from './components/NumberUnitInputSelector';
import GodFire from './assets/icons/godFire.svg';

interface LiquidityAPYEstimate {
	estimate: number;
}

const ProvideLiquidity: React.FunctionComponent = () => {
	const { register, watch, setValue, getValues, setError, handleSubmit } =
		useForm({});
	const [isOpen, toggle] = useToggle();
	const [liquidityAPYEstimate, setLiquidityAPYEstimate] = useState<
		LiquidityAPYEstimate | undefined
	>(undefined);
	const sureSdk = useSureSdk();
	const [pool] = usePool();
	const wallet = useWallet();
	const marketSelectorRef = useRef<HTMLDivElement>(null);

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
		if (pool && sureSdk) {
			const tokenMint = pool.tokenMint;
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
				<div className="action-container-inner-content">
					<p className="p--margin-s p--large p--white ">Provide Liquidity</p>
					<form
						onSubmit={handleSubmit(onSubmit)}
						className={'action-container-inner-content--form'}
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
						</div>

						<div className="action-container-inner-content--row">
							<div className="action-container-inner-content--item">
								<p className="p--margin-xs p--small">Range Start</p>
								<NumberUnitInputSelector
									name="rangeStart"
									valueName="bp"
									register={register}
									setValue={setValue}
									getValues={getValues}
									validateOnBlur={(e) => {
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
									}}
								/>
							</div>

							<div className="action-container-inner-content--item">
								<p className="p--margin-xs p--small">Range End</p>
								<NumberUnitInputSelector
									name="rangeEnd"
									valueName="bp"
									register={register}
									setValue={setValue}
									getValues={getValues}
									validateOnBlur={(e) => {
										const newValueInt = parseInt(e.target.value);
										const newValue = newValueInt - (newValueInt % 10);
										setValue('rangeEnd', newValue);
									}}
								/>
							</div>
						</div>
						{isOpen && <SearchMarket parentRef={marketSelectorRef} />}
						{liquidityAPYEstimate?.estimate && (
							<div className="action-container-inner-content--row_centered">
								<div className="action-container-inner-content--item">
									<h3 className="h3--white h3--center h3--margin-s">
										Estimated APY: 10.2%
									</h3>
									<p className="p--margin-s p--medium p--center">
										Pool APY: 10%
									</p>
									<p className="p--margin-s p--small p--center">
										Premium APY: 0.2%
									</p>
								</div>
							</div>
						)}
						<div className="action-container-inner-content--row_centered">
							<div className="action-container-inner-content--item">
								<MainButton>
									<h3 className="p--white p--margin-0">Provide Liquidity</h3>
								</MainButton>
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
				src={GodFire}
				alt={'god arrow'}
			/>
		</div>
	);
};

export default ProvideLiquidity;
