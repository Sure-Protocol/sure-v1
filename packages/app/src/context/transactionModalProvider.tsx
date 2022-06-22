import React, { createContext, useContext, useState } from 'react';

type TransactionModalType = [boolean, (isOpen: boolean) => void];

const TransactionModalContext = createContext<TransactionModalType>([
	false,
	() => {},
]);

export const TransactionModalProvider: React.FunctionComponent<{
	children: JSX.Element;
}> = ({ children }) => {
	const [isTransactionModalOpen, setIsTransactionModalOpen] =
		useState<TransactionModalType[0]>(false);

	return (
		<TransactionModalContext.Provider
			value={[isTransactionModalOpen, setIsTransactionModalOpen]}
		>
			{children}
		</TransactionModalContext.Provider>
	);
};

export const useTransactionModal = (): TransactionModalType => {
	return useContext(TransactionModalContext);
};
