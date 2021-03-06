import React from 'react';
import { css } from '@emotion/css';
import { theme } from './Themes';
import { useWatch } from 'react-hook-form';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
type Props = {
	children: JSX.Element;
	onClick?: () => void;
	isSubmit?: boolean;
};

const MainButton: React.FunctionComponent<Props> = ({
	children,
	onClick = () => {},
	isSubmit = true,
}) => {
	const wallet = useWallet();
	return (
		<>
			{wallet.connected ? (
				<button
					type={isSubmit ? 'submit' : 'button'}
					className={css`
						background-color: ${theme.colors.surePurple};
						border-radius: 10px;
						border-width: 0;
						padding: 10px;
						padding-left: 5rem;
						padding-right: 5rem;
						width: fit-content;
						cursor: pointer;

						&:hover {
							background-color: ${theme.colors.sureDarkPuprle};
						}
					`}
					onClick={() => onClick()}
				>
					{children}
				</button>
			) : (
				<WalletMultiButton />
			)}
		</>
	);
};

export default MainButton;
