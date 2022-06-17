export class SureDate extends Date {
	static new(ms: number): SureDate {
		return new SureDate(ms);
	}

	getTimeInSeconds(): number {
		return this.getTime() / 1000;
	}

	addHours(hours: number): SureDate {
		return new SureDate(this.setTime(this.getTime() + 60 * 60 * 1000 * hours));
	}
}

export const getUnixTime = () => {
	return new Date().valueOf() / 1000;
};
