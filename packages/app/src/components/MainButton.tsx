import React from 'react';
import { css } from '@emotion/css';
import { theme } from './Themes';
type Props = {
	children: JSX.Element;
};

const MainButton: React.FunctionComponent<Props> = ({ children }) => {
	return (
		<button
			type="submit"
			className={css`
				background-color: ${theme.colors.surePurple};
				border-radius: 10px;
				border-width: 0;
				padding: 10px;
				padding-left: 5rem;
				padding-right: 5rem;
				width: fit-content;
				cursor: pointer;

				&:hover {
					background-color: ${theme.colors.sureDarkPuprle};
				}
			`}
		>
			{children}
		</button>
	);
};

export default MainButton;
