import * as anchor from '@project-serum/anchor';
import { getMint } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
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

	public setAmount(amount: number): Money {
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

	static async convertBNFromDecimals(
		connection: anchor.web3.Connection,
		amount: anchor.BN,
		mint: PublicKey
	): Promise<string> {
		const mintInfo = await getMint(connection, mint);
		const base = new anchor.BN(10 ** mintInfo.decimals);
		let fraction = amount.mod(base).toString(10);
		while (fraction.length < mintInfo.decimals) {
			fraction = `0${fraction}`;
		}
		const whole = amount.div(base).toString(10);
		return `${whole}${fraction == '0' ? '' : `.${fraction}`}`;
	}

	static async convertBNToDecimals(
		connection: anchor.web3.Connection,
		amount: anchor.BN,
		mint: PublicKey
	): Promise<anchor.BN> {
		const mintInfo = await getMint(connection, mint);
		return amount.mul(new anchor.BN(10 ** mintInfo.decimals));
	}
}
