import InfoBox from '../InfoBox';
import PopupBox from './PopupBox';

const BuyCoverageAmountContent: React.FunctionComponent<{
	toggle: () => void;
}> = ({ toggle }) => {
	return (
		<PopupBox title="Protocol to insure" toggle={toggle}>
			<div>
				<p className="p--small p--white p--margin-s">
					The following information about the protocol is needed to buy
					insurance{' '}
				</p>
				<p className="p--small p--white p--margin-0">
					<span className="text--color__pink">Note:</span> Amount can be changed
					at any time
				</p>

				<ul>
					<li>
						<p className="p--small p--white p--margin-0">
							<span className="text--font__bold">Pool</span> - Insurance pool.
							The protocol specifies the smart contract to insure against. The
							token is the denomination of the pool.{' '}
						</p>
					</li>
					<li>
						<p className="p--small p--white p--margin-0">
							<span className="text--font__bold">Amount</span> - The amount to
							insure. In case of a breach you will receive breach% x amount.
						</p>
					</li>
					<li>
						<p className="p--small p--white p--margin-0">
							<span className="text--font__bold">Expiry</span> - The expiration
							date of your contract. Used to calculate the up front premium.
						</p>
					</li>
				</ul>
			</div>
		</PopupBox>
	);
};

export default BuyCoverageAmountContent;
