import React from 'react';

type Props = {
	children: JSX.Element;
};

const MainButton: React.FunctionComponent<Props> = ({ children }) => {
	return <button className="sure-main-button">{children}</button>;
};

export default MainButton;
