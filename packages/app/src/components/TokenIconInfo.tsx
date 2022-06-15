import { css, cx } from '@emotion/css';
import { TokenInfo } from '@solana/spl-token-registry';
import { PublicKey } from '@solana/web3.js';
import { cutString } from '../utils/string';
import { theme } from './Themes';
import TokenIcon from './TokenIcon';

const styles = ({ isVisible }: { isVisible: boolean }) => css`
	display: flex;
	align-items: center;
	background-color: ${theme.colors.sureBlue4};
	color: ${theme.colors.sureWhite};
	border-radius: 5px;
	border-width: 1px;
	justify-content: center;
	padding-left: 5px;
	padding-right: 5px;
	filter: opacity(15%)
		${isVisible &&
		`
    filter: opacity(100%)
    `};
`;

const TokenIconInfo: React.FunctionComponent<{
	token: TokenInfo;
	isVisible?: boolean;
}> = ({ token, isVisible = true }) => {
	return (
		<div className={styles({ isVisible })}>
			<TokenIcon tokenAddress={new PublicKey(token?.address)} />
			<p className="p--margin-0 p--small">
				{token?.name !== '' ? cutString(token?.symbol, 5) : 'USDC'}
			</p>
		</div>
	);
};

export default TokenIconInfo;
