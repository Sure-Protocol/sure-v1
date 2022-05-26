import React from 'react';
import { css, cx } from '@emotion/css';

const theme = {
	colors: {
		sureBlue: '#0C1E7F',
		sureBlue2: '#05152F',
		sureBlue3: 'rgba(12, 30, 127,0.4)',
		sureBlue4: '#324F7E',
	},
};

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
			<div className="sure-info-box--title">
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
