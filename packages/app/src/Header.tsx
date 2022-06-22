import { css } from '@emotion/css';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import SureLogoSlogan from './assets/icons/sureLogoSlogan.svg';
import { SureWalletButton } from './components/SureWalletButton';
function Header() {
	return (
		<div className="header">
			<div className="header-navbar">
				<div className="header-navbar--item">
					<img
						className={css`
							width: 150px;
							height: 150px;
						`}
						src={SureLogoSlogan}
						alt={'god arrow'}
					/>
				</div>
				<div className="header-navbar--item">
					<div className="sure-wallet">
						<SureWalletButton />
					</div>
				</div>
			</div>
		</div>
	);
}

export default Header;
