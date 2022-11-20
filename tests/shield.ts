import { assert } from 'chai';
import * as anchor from '@project-serum/anchor';
import { Shield } from '../target/types/Shield';
describe('Shield', () => {
	const provider = anchor.AnchorProvider.env();
	anchor.setProvider(provider);
	const program = anchor.workspace.Shield as Program<Shield>;
	it('Initialize Shield market', async () => {});
});
