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
				<a
					href={explorerLink(tokenAddress)}
					className={css`
						width: 32px;
						height: 32px;
					`}
				>
					<img
						height={'32px'}
						src={tokens.get(tokenAddress?.toBase58())?.logoURI}
					/>
				</a>
			)}
		</>
	);
};

export default TokenIcon;
