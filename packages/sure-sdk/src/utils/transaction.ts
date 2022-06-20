import * as anchor from '@project-serum/anchor';
import { TransactionSignature } from '@solana/web3.js';
import { getUnixTime } from './time';

async function getTxConfirmation(
	connection: anchor.web3.Connection,
	timeout: number,
	txId: TransactionSignature
) {
	let attemptFinished = false;
	// timeout the method
	setTimeout(() => {
		if (attemptFinished) {
			return;
		}
		attemptFinished = true;
		throw new Error('timeout');
	}, timeout);

	// wait for transaction confirmation
	while (!attemptFinished) {
		// Run each confirmation task async in case some calls crashes or stalls
		(async () => {
			try {
				const signatureStatuses = await connection.getSignatureStatuses([txId]);
				const signatureStatus = signatureStatuses.value[0];

				if (signatureStatus) {
					// If error
					if (signatureStatus.err) {
						attemptFinished = true;
						throw new Error(
							'could not get signature status. cause: ' + signatureStatus.err
						);
					}

					if (signatureStatus.confirmationStatus === 'confirmed') {
						attemptFinished = true;
					}
				}
			} catch (err) {
				console.log('unexpected confirmation result.');
				throw new Error(err);
			}
		})();

		// Timeout
		await new Promise((resolve) => setTimeout(resolve, 500));
	}
}

async function sendAndConfirm(
	connection: anchor.web3.Connection,
	signedTx: anchor.web3.Transaction
) {
	let txConfirmed = false;
	const timeout = 15 * 1000;
	const txStart = getUnixTime();

	// Start sending transaction until confirmation
	let txId = await connection.sendRawTransaction(signedTx.serialize(), {
		skipPreflight: true,
	});

	(async () => {
		while (!txConfirmed && getUnixTime() - txStart < timeout) {
			await connection.sendRawTransaction(signedTx.serialize(), {
				skipPreflight: true,
			});
			await new Promise((resolve) => setTimeout(resolve, 1000));
		}
	})();

	// Try to get confirmation within timeout
	try {
		await getTxConfirmation(connection, timeout, txId);
	} catch (err) {
		txConfirmed = true;
		throw new Error(err);
	} finally {
		txConfirmed = true;
	}

	return txId;
}
export async function sendTransaction(
	connection: anchor.web3.Connection,
	tx: anchor.web3.Transaction,
	signer: anchor.Wallet
): Promise<string> {
	// tx global vars

	// Prepare transaction
	tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
	tx.feePayer = signer.publicKey;
	const signedTx = await signer.signTransaction(tx);
	try {
		const txId = await sendAndConfirm(connection, signedTx);
		return txId;
	} catch (err) {
		console.error(err);
		throw new Error('transaction.sendTransaction.failed. cause: ' + err);
	}
}
