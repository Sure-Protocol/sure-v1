import down from './assets/icons/expand_more.svg';
import MainButton from './components/MainButton';
import InfoBox from './components/InfoBox';
import { TokensContext } from './context/tokens';
import { TokenListProvider } from '@solana/spl-token-registry';
import { useContext } from 'react';
import { SelectMarket } from './components/SelectMarket';
import DateSelector from './components/DateSelector';

const BuyInsurance = () => {
	return (
		<div className="action-container">
			<div className="action-container-inner">
				<div className="sure-buy-insurance-container">
					<p className="p--margin-s">Coverage position</p>
					<div className="sure-buy-insurance-selectors--horisontal">
						<SelectMarket />
						<div className="sure-buy-insurance-selector--date">
							<DateSelector />
						</div>
					</div>
					<p className="p--small p--margin-s">
						Available liquidity 100,000 USDC
					</p>
				</div>
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

				<div className="sure-buy-insurance-container--centered">
					<p className="p--margin-s p--medium p--center">Estimated price</p>
					<h3 className="h3--white h3--center h3--margin-s"> 340 USDC</h3>
					<p className="p--margin-s p--small p--center">Premium: 2.4%</p>
				</div>

				<div className="sure-buy-insurance-container--centered">
					<MainButton>
						<h3 className="p--white p--margin-0">Buy</h3>
					</MainButton>
				</div>
			</div>
		</div>
	);
};

export default BuyInsurance;
