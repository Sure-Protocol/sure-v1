import React from 'react';
import exclamation from './../assets/icons/exclamation.svg';
import { css, cx } from '@emotion/css';
import { theme } from './Themes';
interface Props {
	title: string;
	children: JSX.Element;
}

const InfoBox: React.FunctionComponent<Props> = ({ title, children }) => {
	return (
		<div
			className={css`
				background-color: ${theme.colors.sureBlue3};
				border-radius: 10px;
				padding: 10px;
			`}
		>
			<div
				className={css`
					display: flex;
					flex-direction: row;
					align-items: center;
				`}
			>
				<img
					src={exclamation}
					alt="Sure pink exclamation mark"
					className={css`
						padding: 5px;
						height: 18px;
					`}
				/>
				<p className="p--margin-0">{title}</p>
			</div>
			<div
				className={css`
					display: flex;
					justify-content: center;
				`}
			>
				{children}
			</div>
		</div>
	);
};

export default InfoBox;
