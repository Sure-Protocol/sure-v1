import PopupBox from './PopupBox';

const BuyCoverageContent: React.FunctionComponent<{
	toggle: () => void;
}> = ({ toggle }) => {
	return (
		<PopupBox title="Buy Coverage" toggle={toggle}>
			<div>
				<p className="p--small p--white p--margin-s">To buy insurance:</p>
				<p className="p--small p--white p--margin-0">
					<span className="text--color__pink">Note:</span> amount and expiry can
					be changed at any time
				</p>
				<ul>
					<li>
						<p className="p--small p--white p--margin-0">
							Select the protocol you want to insure against
						</p>
					</li>
					<li>
						<p className="p--small p--white p--margin-0">
							Pick amount to insure
						</p>
					</li>
					<li>
						<p className="p--small p--white p--margin-0">
							Choose the expiration date for the policy
						</p>
					</li>
				</ul>
			</div>
		</PopupBox>
	);
};

export default BuyCoverageContent;
