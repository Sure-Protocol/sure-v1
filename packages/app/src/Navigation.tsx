import Header from './Header';
import { Routes, Route, Link, NavLink } from 'react-router-dom';
import BuyInsurance from './BuyInsurance';
import ActionBar from './ActionBar';
import { ManageMarkets } from './ManageMarkets';
import { InsuranceContractProvider } from './context/insuranceContract';
import { PoolProvider } from './context/surePool';
import { SearchProvider } from './context/searchToggle';
import ProvideLiquidity from './ProvideLiquidity';

const Navigation = () => {
	return (
		<>
			<Header />
			<div className="sure-page">
				<div className="container">
					<ActionBar />
				</div>
				<div className="container">
					<Routes>
						<Route
							path="/"
							element={
								<PoolProvider>
									<InsuranceContractProvider>
										<SearchProvider>
											<BuyInsurance />
										</SearchProvider>
									</InsuranceContractProvider>
								</PoolProvider>
							}
						/>
						<Route
							path="/liquidity"
							element={
								<PoolProvider>
									<SearchProvider>
										<ProvideLiquidity />
									</SearchProvider>
								</PoolProvider>
							}
						/>
						<Route path="/markets" element={<ManageMarkets />} />
					</Routes>
				</div>
			</div>
		</>
	);
};

export default Navigation;
