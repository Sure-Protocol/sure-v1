import { PublicKey } from '@solana/web3.js';
import { useTokens } from '../context/tokens';
import { css } from '@emotion/css';
import { explorerLink } from '../utils/links';
import GenerateToken from '../assets/icons/generateToken.svg';
import { prettyPublicKey, prettyPublicKeyString } from '../utils/publickey';

interface Props {
	tokenAddress: string;
}

const TokenIcon: React.FunctionComponent<Props> = ({ tokenAddress }) => {
	const tokens = useTokens();
	const token = tokens.get(tokenAddress);
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
					{token?.logoURI ? (
						<img
							height={'32px'}
							width={'32px'}
							className={css`
								border-radius: 100%;
							`}
							src={token.logoURI}
						/>
					) : (
						<div>
							<img
								height={'32px'}
								width={'32px'}
								className={css`
									border-radius: 100%;
								`}
								src={GenerateToken}
							/>
							<p className="p--small">{prettyPublicKeyString(tokenAddress)}</p>
						</div>
					)}
				</div>
			)}
		</>
	);
};

export default TokenIcon;
