import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';

function Header() {
	return (
		<div className="header">
			<div className="header-navbar">
				<div className="header-navbar-item">
					<div className="sure-wallet">
						<WalletMultiButton className="sure-wallet-button" />
					</div>
				</div>
			</div>
		</div>
	);
}

export default Header;
