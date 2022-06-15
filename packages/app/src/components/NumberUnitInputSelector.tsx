import { css, cx } from '@emotion/css';
import React from 'react';
import {
	FieldValues,
	UseFormGetValues,
	UseFormRegister,
	UseFormSetValue,
} from 'react-hook-form';
import { theme } from './Themes';

interface NumberUnitInputSelectorProps {
	name: string;
	valueName: string;
	register: UseFormRegister<FieldValues>;
	setValue: UseFormSetValue<FieldValues>;
	getValues: UseFormGetValues<FieldValues>;
	validateOnBlur?: (e: any) => void;
}

const NumberUnitInputSelector: React.FunctionComponent<
	NumberUnitInputSelectorProps
> = ({ name, valueName, register, setValue, getValues, validateOnBlur }) => {
	return (
		<div
			className={css`
				//
				border-radius: 5px;
				flex-grow: 2;
				background-color: ${theme.colors.sureBlue4};
				padding: 4px;
				display: flex;
				flex-direction: row;
				align-items: center;
			`}
		>
			<input
				{...register(name, {
					min: 0,
					max: 10000,
					valueAsNumber: true,
					onBlur: (e) => validateOnBlur?.(e),
				})}
				placeholder="0"
				className={cx(
					'input-number-field',
					css`
						text-align: center;
					`
				)}
			/>
			<p className="p--margin-0 p-margin-center">{valueName}</p>
		</div>
	);
};

export default NumberUnitInputSelector;
