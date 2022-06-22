import React, { FC, useMemo } from 'react';
import './styles/index.scss';
import '@solana/wallet-adapter-react-ui/styles.css';
import {
	ConnectionProvider,
	WalletProvider,
} from '@solana/wallet-adapter-react';
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import {
	PhantomWalletAdapter,
	Coin98WalletAdapter,
	GlowWalletAdapter,
	SolletWalletAdapter,
} from '@solana/wallet-adapter-wallets';
import {
	WalletModalProvider,
	WalletDisconnectButton,
	WalletMultiButton,
} from '@solana/wallet-adapter-react-ui';
import { clusterApiUrl } from '@solana/web3.js';
import {
	BrowserRouter,
	BrowserRouter as Router,
	Route,
	Routes,
} from 'react-router-dom';
import BuyInsurance from './BuyInsurance';
import Navigation from './Navigation';
import { TokensProvider } from './context/tokens';
import { SureSdkProvider } from './context/sureSdk';
import { SurePoolsProvider } from './context/surePools';
import { TransactionHistoryProvider } from './context/transactionHistory';
import TransactionModal from './components/TransactionModal';
import { TransactionModalProvider } from './context/transactionModalProvider';
import { LoadingProvider } from './context/loadingProvider';

// Default styles that can be overridden by your app
//require('@solana/wallet-adapter-react-ui/styles.css');

const App: FC = () => {
	// The network can be set to 'devnet', 'testnet', or 'mainnet-beta'.
	const network = WalletAdapterNetwork.Devnet;

	// You can also provide a custom RPC endpoint.
	const endpoint = useMemo(() => clusterApiUrl(network), [network]);

	// @solana/wallet-adapter-wallets includes all the adapters but supports tree shaking and lazy loading --
	// Only the wallets you configure here will be compiled into your application, and only the dependencies
	// of wallets that your users connect to will be loaded.
	const wallets = useMemo(
		() => [
			new PhantomWalletAdapter(),
			new Coin98WalletAdapter(),
			new GlowWalletAdapter(),
		],
		[network]
	);

	return (
		<BrowserRouter>
			<ConnectionProvider endpoint={endpoint}>
				<WalletProvider wallets={wallets} autoConnect>
					<WalletModalProvider>
						<LoadingProvider>
							<SureSdkProvider>
								<SurePoolsProvider>
									<TokensProvider>
										<TransactionHistoryProvider>
											<TransactionModalProvider>
												<div>
													<Navigation />
													<TransactionModal />
												</div>
											</TransactionModalProvider>
										</TransactionHistoryProvider>
									</TokensProvider>
								</SurePoolsProvider>
							</SureSdkProvider>
						</LoadingProvider>
					</WalletModalProvider>
				</WalletProvider>
			</ConnectionProvider>
		</BrowserRouter>
	);
};

export default App;
