import { Popover } from 'react-tiny-popover';
import _ from 'lodash';
import BuyCoverageContent from './BuyCoverageContent';
import WithExclamation from '../WithExclamation';
import React from 'react';

const TitleWithPopover: React.FunctionComponent<{
	children: JSX.Element;
	isOpen: boolean;
	Content: React.FunctionComponent<{ toggle: () => void }>;
	toggle: (open: boolean) => void;
}> = ({ children, Content, isOpen, toggle }) => {
	return (
		<Popover
			isOpen={isOpen}
			onClickOutside={() => toggle(false)}
			positions={['top']} // preferred positions by priority
			align={'start'}
			content={<Content toggle={() => toggle(true)} />}
		>
			<WithExclamation onClick={() => toggle(true)} left={false}>
				{children}
			</WithExclamation>
		</Popover>
	);
};

export default TitleWithPopover;
