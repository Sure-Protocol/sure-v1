import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';

function Header() {
	return (
		<div className="header">
			<div className="header-navbar">
				<div className="header-navbar-item">
					<WalletMultiButton />
				</div>
			</div>
		</div>
	);
}

export default Header;
