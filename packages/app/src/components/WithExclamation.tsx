import { css } from '@emotion/css';
import React, { LegacyRef } from 'react';
import Exclamation from './../assets/icons/exclamation.svg';

const WithExclamation = React.forwardRef<
	HTMLDivElement,
	{ children: JSX.Element; onClick?: () => void; left?: boolean }
>(({ children, onClick = () => {}, left = true }, ref) => {
	return (
		<div
			className={css`
				display: flex;
				flex-direction: row;
				width: fit-content;
				align-items: center;
				justify-content: center;
			`}
		>
			{left && (
				<div
					className={css`
						display: flex;
						align-items: center;
					`}
					ref={ref}
				>
					<img
						className={css`
							padding-left: 5px;
							padding-right: 5px;
						`}
						src={Exclamation}
						alt={'Information icon'}
						onClick={() => onClick()}
					/>
				</div>
			)}
			{children}
			{!left && (
				<div
					className={css`
						display: flex;
						align-items: center;
					`}
					ref={ref}
				>
					<img
						className={css`
							padding-left: 5px;
							padding-right: 5px;
						`}
						src={Exclamation}
						alt={'Information icon'}
						onClick={() => onClick()}
					/>
				</div>
			)}
		</div>
	);
});

export default WithExclamation;
