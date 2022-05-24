import { useState } from 'react';
import * as solanaReactUI from '@solana/wallet-adapter-react-ui';
import * as solanaAdapter from '@solana/wallet-adapter-react';

function App() {
	const [count, setCount] = useState(0);
	return (
		<div className="App">
			<header className="App-header">
				<solanaReactUI.WalletConnectButton />
			</header>
		</div>
	);
}

export default App;
