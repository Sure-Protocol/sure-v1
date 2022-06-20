export class SureErrors {
	readonly name;
	static NotEnoughLiquidity = new SureErrors(
		'Not enough liquidity to carry out computation'
	);
	static CouldNotGetAccount = new SureErrors('Could not get account');
	static Default = new SureErrors('Default Error');
	constructor(name) {
		this.name = name;
	}
}

export class SureError extends Error {
	readonly error;

	constructor(message: string, error: SureErrors) {
		super(message);
		this.message = message;
		this.error = error;
	}
}
