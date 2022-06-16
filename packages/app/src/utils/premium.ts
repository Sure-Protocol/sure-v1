export const prettyPrintPremium = (premium: number): string => {
	if (premium < 0) {
		return 'inf ';
	}
	return premium.toString();
};
