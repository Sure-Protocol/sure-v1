import PopupBox from './PopupBox';

const BuyCoverageExpiryContent: React.FunctionComponent<{
	toggle: () => void;
}> = ({ toggle }) => {
	return (
		<PopupBox title="Coverage Expiry" toggle={toggle}>
			<div>
				<p className="p--small p--white p--margin-s">
					Choose the expiration of the insurance contract.
				</p>
				<p className="p--small p--white p--margin-0">
					<span className="text--color__pink">Note:</span> expiry can be changed
					at any time
				</p>
				<p className="p--small p--white p--margin-s">
					The expiration date is used to calculate the premium you have to pay
					up front. By choosing a small period you post a low premium, but you
					have to come back to extend the contract every time it is about to
					expire.{' '}
				</p>
			</div>
		</PopupBox>
	);
};

export default BuyCoverageExpiryContent;
