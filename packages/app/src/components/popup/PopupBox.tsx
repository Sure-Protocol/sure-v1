import { css } from '@emotion/css';
import { theme } from '../Themes';
import Exclamation from './../../assets/icons/exclamation.svg';

const PopupBox: React.FunctionComponent<{
	title: string;
	children: JSX.Element;
	toggle: () => void;
}> = ({ title, children, toggle }) => {
	return (
		<div
			className={css`
				background-color: ${theme.colors.sureBlue2};
				border-radius: 10px;
				border-style: solid;
				border-width: 1px;
				border-color: ${theme.colors.surePink};
				padding: 10px;
				padding-right: 20px;
				padding-left: 20px;
				width: 300px;
				color: ${theme.colors.sureWhite};
				z-index: 100;
			`}
		>
			<div
				className={css`
					display: flex;
					flex-direction: row;
				`}
			>
				<img
					className="text--margin-right__small"
					src={Exclamation}
					alt={'Information image'}
					onClick={toggle}
				/>
				<h4 className="text--margin-vertical__xsmall text--color__white ">
					{title}
				</h4>
			</div>

			{children}
		</div>
	);
};

export default PopupBox;
