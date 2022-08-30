export const saveSalt = (salt: string, name: string) => {
	const key = 'ss-' + name.split(' ').join('-');
	localStorage.setItem(key, salt);
};

export const getSalt = (name: string): string | null => {
	const key = 'ss-' + name.split(' ').join('-');
	return localStorage.getItem(key);
};
