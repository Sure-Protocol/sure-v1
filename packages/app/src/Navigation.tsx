import Header from './Header';
import { Routes, Route, Link, NavLink } from 'react-router-dom';
import BuyInsurance from './BuyInsurance';
import ActionBar from './ActionBar';
import { ManageMarkets } from './ManageMarkets';

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
						<Route path="/" element={<BuyInsurance />} />
						<Route path="/markets" element={<ManageMarkets />} />
					</Routes>
				</div>
			</div>
		</>
	);
};

export default Navigation;
