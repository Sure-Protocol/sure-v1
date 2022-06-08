import * as anchor from '@project-serum/anchor';
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

	convertToDecimals(): anchor.BN {
		const amountBN = new anchor.BN(this.amount);
		const decimalsBN = new anchor.BN(10).pow(new anchor.BN(this.decimals));
		return amountBN.mul(decimalsBN);
	}

	convertToAmount(amountDecimals: anchor.BN): void {
		const decimalsBN = new anchor.BN(10).pow(new anchor.BN(this.decimals));
		this.amount = amountDecimals.div(decimalsBN).toNumber();
	}
}
