import * as anchor from '@project-serum/anchor';

//TODO Write unit tests

export type BitmapType = {
	bump: number;
	wordPos: number;
	spacing: number;
	word: anchor.BN[];
};

export class Bitmap {
	protected bump: number;
	protected wordPos: number;
	spacing: number;
	protected word: anchor.BN[];

	constructor(
		bump: number,
		wordPos: number,
		spacing: number,
		word: anchor.BN[]
	) {
		this.bump = bump;
		this.wordPos = wordPos;
		this.spacing = spacing;
		this.word = word;
	}

	static new(bitmap: BitmapType): Bitmap {
		return new Bitmap(bitmap.bump, bitmap.wordPos, bitmap.spacing, bitmap.word);
	}

	getLowestBit(): number {
		const u256 = this.word.flatMap((word) => {
			return word.toString(2, 64).split('').reverse().join('');
		})[0];

		return u256.indexOf('1');
	}

	getHighestBit(): number {
		const u256 = this.word.flatMap((word) => {
			return word.toString(2, 64).split('').reverse().join('');
		})[0];

		return u256.lastIndexOf('1');
	}

	getTickFromBit(bit: number): number {
		return 0 + this.spacing * bit;
	}

	getBitFromTick(tick: number): number {
		return tick / this.spacing;
	}

	getLowestTick(): number {
		const lowestBit = this.getLowestBit();
		if (lowestBit === -1) {
			return -1;
		}
		return this.getTickFromBit(lowestBit);
	}

	getHighestTick(): number {
		const highestBit = this.getHighestBit();
		if (highestBit === -1) {
			return -1;
		}
		return this.getTickFromBit(highestBit);
	}

	getNextTick(tick: number): number {
		const bit = this.getBitFromTick(tick);

		const u256 = this.word.flatMap((word) => {
			return word.toString(2, 64).split('').reverse().join('');
		})[0];

		const remainingBitmap = u256.slice(bit + 1);
		const subBit = remainingBitmap.indexOf('1');
		if (subBit === -1) {
			return -1;
		}
		const nextBit = subBit + bit + 1;

		return this.getTickFromBit(nextBit);
	}

	getPreviousTick(tick: number): number {
		const bit = this.getBitFromTick(tick);

		const u256 = this.word.flatMap((word) => {
			return word.toString(2, 64).split('').reverse().join('');
		})[0];
		const priorBitmap = u256.slice(0, bit);
		const lastBit = priorBitmap.lastIndexOf('1');
		if (lastBit === -1) {
			return lastBit;
		}

		return this.getTickFromBit(lastBit);
	}
}
