import * as anchor from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';
import { Money } from '@surec/sdk';
import { useEffect, useState } from 'react';
import { useSureSdk } from '../context/sureSdk';

interface EstimateInsuranceProps {
	amount: number;
	tokenMint: PublicKey;
	pool: PublicKey;
}

const EstimateInsurance: React.FunctionComponent<EstimateInsuranceProps> = ({
	amount,
	tokenMint,
	pool,
}) => {
	const sureSdk = useSureSdk();
	const [estimate, setEstimate] = useState(['', '']);
	useEffect(() => {
		const estimateYearlyPremium = async () => {
			const estimate = await sureSdk?.insurance.estimateYearlyPremium(
				amount,
				tokenMint,
				pool
			);
			if (estimate) {
				setEstimate([estimate[0], estimate[1]]);
			}
		};
		estimateYearlyPremium();
	});
	return (
		<div className="sure-buy-insurance-container--centered">
			<p className="p--margin-s p--medium p--center">Estimated price</p>
			<h3 className="h3--white h3--center h3--margin-s">{`${estimate[0]} USDC`}</h3>
			<p className="p--margin-s p--small p--center">Premium: 2.4%</p>
		</div>
	);
};

export default EstimateInsurance;
