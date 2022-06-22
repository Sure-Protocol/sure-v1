import { css } from '@emotion/css';
import { useWallet } from '@solana/wallet-adapter-react';
import { useWalletModal } from '@solana/wallet-adapter-react-ui';
import {
	WalletConnectButton,
	WalletIcon,
	WalletModalButton,
} from '@solana/wallet-adapter-react-ui';
import React, {
	useCallback,
	useEffect,
	useMemo,
	useRef,
	useState,
} from 'react';
import { useTransactionModal } from '../context/transactionModalProvider';

export const SureWalletButton: React.FunctionComponent<{}> = ({}) => {
	const { publicKey, wallet, disconnect } = useWallet();
	const { setVisible } = useWalletModal();
	const [copied, setCopied] = useState(false);
	const [active, setActive] = useState(false);
	const ref = useRef<HTMLUListElement>(null);
	const [isTransactionModalOpen, setIsTransactionModalOpen] =
		useTransactionModal();

	const base58 = useMemo(() => publicKey?.toBase58(), [publicKey]);
	const content = useMemo(() => {
		if (!wallet || !base58) return null;
		return base58.slice(0, 4) + '..' + base58.slice(-4);
	}, [wallet, base58]);

	const copyAddress = useCallback(async () => {
		if (base58) {
			await navigator.clipboard.writeText(base58);
			setCopied(true);
			setTimeout(() => setCopied(false), 400);
		}
	}, [base58]);

	const openDropdown = useCallback(() => {
		setActive(true);
	}, []);

	const closeDropdown = useCallback(() => {
		setActive(false);
	}, []);

	const openModal = useCallback(() => {
		setVisible(true);
		closeDropdown();
	}, [closeDropdown]);

	const openTransactionModal = useCallback(() => {
		setIsTransactionModalOpen(true);
		closeDropdown();
	}, []);

	useEffect(() => {
		const listener = (event: MouseEvent | TouchEvent) => {
			const node = ref.current;

			// Do nothing if clicking dropdown or its descendants
			if (!node || node.contains(event.target as Node)) return;

			closeDropdown();
		};

		document.addEventListener('mousedown', listener);
		document.addEventListener('touchstart', listener);

		return () => {
			document.removeEventListener('mousedown', listener);
			document.removeEventListener('touchstart', listener);
		};
	}, [ref, closeDropdown]);

	if (!wallet) return <WalletModalButton />;
	if (!base58) return <WalletConnectButton />;

	return (
		<div
			className={css`
				position: relative;
				display: inline-block;
			`}
		>
			<button
				aria-expanded={active}
				className={css`
					background-color: transparent;
					border: none;
					color: #fff;
					cursor: pointer;
					display: flex;
					align-items: center;
					font-family: 'DM Sans', 'Roboto', 'Helvetica Neue', Helvetica, Arial,
						sans-serif;
					font-size: 16px;
					font-weight: 600;
					height: 48px;
					line-height: 48px;
					padding: 0 24px;
					border-radius: 4px;
				`}
				style={{ pointerEvents: active ? 'none' : 'auto' }}
				onClick={openDropdown}
			>
				<i className="wallet-adapter-button-start-icon">
					<WalletIcon wallet={wallet} />
				</i>
				{content}
			</button>
			<ul
				aria-label="dropdown-list"
				className={`wallet-adapter-dropdown-list ${
					active && 'wallet-adapter-dropdown-list-active'
				}`}
				ref={ref}
				role="menu"
			>
				<li
					onClick={copyAddress}
					className="wallet-adapter-dropdown-list-item"
					role="menuitem"
				>
					{copied ? 'Copied' : 'Copy address'}
				</li>
				<li
					onClick={openModal}
					className="wallet-adapter-dropdown-list-item"
					role="menuitem"
				>
					Change wallet
				</li>
				<li
					onClick={openTransactionModal}
					className="wallet-adapter-dropdown-list-item"
					role="menuitem"
				>
					Transactions
				</li>
				<li
					onClick={disconnect}
					className="wallet-adapter-dropdown-list-item"
					role="menuitem"
				>
					Disconnect
				</li>
			</ul>
		</div>
	);
};
