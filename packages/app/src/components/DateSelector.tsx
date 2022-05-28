import { css } from '@emotion/css';
import { theme } from './Themes';

const DateSelector = () => {
	return (
		<input
			type="date"
			className={css`
				background-color: transparent;
				border-radius: 5px;
				border-width: 1px;
				border-color: transparent;
				padding: 5px;
				width: fit-content;
				text-align: center;
				color: ${theme.colors.sureWhite};
				&:focus {
					outline: none;
				}
			`}
			placeholder="10.August 2022"
		/>
	);
};

export default DateSelector;
