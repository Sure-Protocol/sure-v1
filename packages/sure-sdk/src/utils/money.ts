export class Money {
	protected amount: number;

	protected decimals: number;

	constructor(decimals: number, amount: number) {
		this.amount = amount;
		this.decimals = decimals;
	}

	static new(decimals: number, amount: number): Money {
		return new Money(decimals, amount);
	}

	setAmount(amount: number): Money {
		this.amount = amount;
		return this;
	}

	convertToDecimals(): number {
		return this.amount * 10 ** this.decimals;
	}

	convertToAmount(amountDecimals: number): void {
		this.amount = amountDecimals / 10 ** this.decimals;
	}
}
