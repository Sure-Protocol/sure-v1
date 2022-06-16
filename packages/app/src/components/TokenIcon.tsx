import { PublicKey } from '@solana/web3.js';
import { useTokens } from '../context/tokens';
import { css } from '@emotion/css';
import { explorerLink } from '../utils/links';

interface Props {
	tokenAddress: PublicKey;
}

const TokenIcon: React.FunctionComponent<Props> = ({ tokenAddress }) => {
	const tokens = useTokens();
	return (
		<>
			{tokens && (
				<div
					className={css`
						width: 32px;
						height: 32px;
						margin-right: 4px;
						margin-left: 4px;
					`}
				>
					<img
						height={'32px'}
						width={'32px'}
						className={css`
							border-radius: 100%;
						`}
						src={tokens.get(tokenAddress?.toBase58())?.logoURI}
					/>
				</div>
			)}
		</>
	);
};

export default TokenIcon;
