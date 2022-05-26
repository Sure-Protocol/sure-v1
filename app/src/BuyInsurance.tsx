import down from './assets/icons/expand_more.svg';
import MainButton from './components/MainButton';
import InfoBox from './components/InfoBox';

const BuyInsurance = () => {
	return (
		<div className="action-container">
			<div className="sure-buy-insurance">
				<div className="sure-buy-insurance-container">
					<p className="p--margin-s">Coverage position</p>
					<div className="sure-buy-insurance-selectors--horisontal">
						<div className="sure-buy-insurance-selector--amount">
							<button className="sure-buy-insurance-selector--amount__button">
								<div className="sure-token">sol</div>
								<div className="sure-token--name">
									<p className="p--margin-0 p--white p--bold">SOL</p>
								</div>
								<div className="sure-icon">
									<img src={down} alt="logo" className="icon-small" />
								</div>
							</button>
							<input
								className="sure-buy-insurance-selector--amount__amount"
								placeholder="0.00"
								typeof="decimals"
							/>
							<button className="sure-buy-insurance-selector--amount__token">
								<p className="p--margin-0">USDC</p>
							</button>
						</div>
						<div className="sure-buy-insurance-selector--date">
							<input
								type="date"
								className="sure-buy-insurance-selector--date__select"
								placeholder="10.August 2022"
							/>
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
