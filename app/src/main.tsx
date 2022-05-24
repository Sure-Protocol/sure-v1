import React from 'react';
import ReactDOM from 'react-dom/client';
import { Wallet } from './WalletWrapper';
import App from './App';
ReactDOM.createRoot(document.getElementById('root')!).render(
	<React.StrictMode>
		<Wallet />
	</React.StrictMode>
);
