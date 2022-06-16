import { NavLink } from 'react-router-dom';

// Action bar navigates users to Sure actions
const ActionBar = () => {
	return (
		<div className="sure-navbar">
			<NavLink
				className={({ isActive }) =>
					`sure-navbar-link${isActive ? '--active' : ''}`
				}
				to="/"
			>
				Insurance
			</NavLink>
			<NavLink
				className={({ isActive }) =>
					`sure-navbar-link${isActive ? '--active' : ''}`
				}
				to="/liquidity"
			>
				Liquidity
			</NavLink>
			<NavLink
				className={({ isActive }) =>
					`sure-navbar-link${
						isActive ? '--active' : ''
					} sure-navbar-link--disabled`
				}
				to="/dashboard"
			>
				<p>Dashboard</p>
			</NavLink>
			<NavLink
				className={({ isActive }) =>
					`sure-navbar-link${isActive ? '--active' : ''}`
				}
				to="/pools"
			>
				Pools
			</NavLink>
		</div>
	);
};

export default ActionBar;
