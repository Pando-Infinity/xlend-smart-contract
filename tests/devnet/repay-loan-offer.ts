import {
	Connection,
	Keypair,
	PublicKey,
	SystemProgram,
	clusterApiUrl,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import { AnchorProvider } from '@project-serum/anchor';
import {
	OPERATE_SYSTEM_SECRET_KEY,
	HOT_WALLET_SECRET_KEY,
  LENDER_SECRET_KEY,
  BORROWER_SECRET_KEY,
} from '../accounts';

import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { EnsoLending } from '../../target/types/enso_lending';

import enso_lending_idl from '../../target/idl/enso_lending.json';
import { getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID, NATIVE_MINT } from '@solana/spl-token';
import { generateId } from '../utils';

const enso_lending_idl_string = JSON.stringify(enso_lending_idl);
const enso_lending_idl_obj = JSON.parse(enso_lending_idl_string);
const PROGRAM_ID_DEV_NET = 'BderhzujHHQNjhCZGRjLWnN2XQ66q4EZnZx2p5WLJnBV';

const programId = new PublicKey(PROGRAM_ID_DEV_NET);
const connection = new Connection(clusterApiUrl('devnet'), 'confirmed');

const ownerAccountSetting = Keypair.fromSecretKey(
	Uint8Array.from(OPERATE_SYSTEM_SECRET_KEY)
);
const hotWallet = Keypair.fromSecretKey(Uint8Array.from(HOT_WALLET_SECRET_KEY));
const lender = Keypair.fromSecretKey(Uint8Array.from(LENDER_SECRET_KEY));
const borrower = Keypair.fromSecretKey(Uint8Array.from(BORROWER_SECRET_KEY));

const usdcMintDecimal = 6;
const solDecimal = 9;
const sol_usd_price_feed_id = 'J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix';
const usdc_usd_price_feed_id = '5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7';
const mintSolWrappedAccount = new PublicKey(NATIVE_MINT);

const mintUsdcAccount = new PublicKey(
	'Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr'
);

const providerWallet = new anchor.Wallet(Keypair.generate());

const provider = new anchor.AnchorProvider(
	connection,
	providerWallet,
	AnchorProvider.defaultOptions()
);

const program = new Program<EnsoLending>(
	enso_lending_idl_obj,
	programId,
	provider
);

xdescribe('enso-lending-devnet', () => {
  it('createLoanOffer', async () => {
    const lendAmount = 100 * Math.pow(10, usdcMintDecimal);
    const waitingInterestAmount = 5 * Math.pow(10, usdcMintDecimal);
		const duration = 14;
    const randomId = generateId(10);
		const tierId = 'tier_' + randomId;
		const lenderFeePercent = 0;
		const borrowerFeePercent = 0;

		const lendOfferId = 'lend_offer_' + randomId;
		const loanOfferId = 'loan_offer_' + randomId;
		const interest = 0.05;

		const seedSettingAccount = [
			Buffer.from('enso'),
			Buffer.from('setting_account'),
			Buffer.from(tierId),
			program.programId.toBuffer(),
		];

		const settingAccount = PublicKey.findProgramAddressSync(
			seedSettingAccount,
			program.programId
		)[0];

		const seedLendOffer = [
			Buffer.from('enso'),
			Buffer.from('lend_offer'),
			lender.publicKey.toBuffer(),
			Buffer.from(lendOfferId),
			program.programId.toBuffer(),
		];

		const lendOfferAccount = PublicKey.findProgramAddressSync(
			seedLendOffer,
			program.programId
		)[0];

    const seedLoanOffer = [
			Buffer.from('enso'),
			Buffer.from('loan_offer'),
			borrower.publicKey.toBuffer(),
			Buffer.from(loanOfferId),
			program.programId.toBuffer(),
		];

		const loanOfferAccount = PublicKey.findProgramAddressSync(
			seedLoanOffer,
			program.programId
		)[0];

		const hotWalletUsdcAta = await getOrCreateAssociatedTokenAccount(
			connection,
			ownerAccountSetting,
			mintUsdcAccount,
			hotWallet.publicKey
		);
    
		const lenderOfferAtaUsdc = await getOrCreateAssociatedTokenAccount(
			connection,
			lender,
			mintUsdcAccount,
			lender.publicKey
		);

    const borrowerAtaUsdc = await getOrCreateAssociatedTokenAccount(
			connection,
			borrower,
			mintUsdcAccount,
			borrower.publicKey
		);

    const systemAtaUsdc = await getOrCreateAssociatedTokenAccount(
			connection,
			ownerAccountSetting,
			mintUsdcAccount,
			ownerAccountSetting.publicKey
		);

		const sol_usd_price_feed = new PublicKey(sol_usd_price_feed_id);
		const usdc_usd_price_feed = new PublicKey(usdc_usd_price_feed_id);

		// Create setting account
		const settingAccountTsx = await program.methods
			.initSettingAccount(
				tierId,
				new anchor.BN(lendAmount),
				new anchor.BN(duration),
				lenderFeePercent,
				borrowerFeePercent
			)
			.accounts({
				owner: ownerAccountSetting.publicKey,
				receiver: hotWallet.publicKey,
				settingAccount,
				lendMintAsset: mintUsdcAccount,
				collateralMintAsset: mintSolWrappedAccount,
				systemProgram: SystemProgram.programId,
				collateralPriceFeedAccount: sol_usd_price_feed,
				lendPriceFeedAccount: usdc_usd_price_feed,
			})
			.transaction();

		await sendAndConfirmTransaction(connection, settingAccountTsx, [
			ownerAccountSetting,
		]);

		// Create lend offer account
		const lendOfferTsx = await program.methods
			.createLendOffer(lendOfferId, tierId, interest)
			.accounts({
				hotWalletAta: hotWalletUsdcAta.address,
				lender: lender.publicKey,
				lenderAtaAsset: lenderOfferAtaUsdc.address,
				lendOffer: lendOfferAccount,
				mintAsset: mintUsdcAccount,
				settingAccount,
				tokenProgram: TOKEN_PROGRAM_ID,
				systemProgram: SystemProgram.programId,
			})
			.transaction();

		await sendAndConfirmTransaction(connection, lendOfferTsx, [lender]);

    const collateralAmount = 2 * Math.pow(10, solDecimal); // 2 SOL

    // Borrower create loan offer
    const loanOfferTsx = await program.methods
			.createLoanOfferNative(
				loanOfferId,
				lendOfferId,
				tierId,
				new anchor.BN(collateralAmount)
			)
			.accounts({
				lender: lender.publicKey,
				borrower: borrower.publicKey,
				lendOffer: lendOfferAccount,
				loanOffer: loanOfferAccount,
				collateralPriceFeedAccount: sol_usd_price_feed,
				lendPriceFeedAccount: usdc_usd_price_feed,
				receiver: hotWallet.publicKey,
				settingAccount,
				systemProgram: SystemProgram.programId,
        collateralMintAsset: mintSolWrappedAccount,
        lendMintAsset: mintUsdcAccount,
			})
      .transaction();

      await sendAndConfirmTransaction(connection, loanOfferTsx, [borrower]);

    // System update loan offer
    const systemUpdateLoanOfferTsx = await program.methods
			.systemUpdateLoanOffer(loanOfferId, tierId, new anchor.BN(lendAmount))
			.accounts({
				mintAsset: mintUsdcAccount,
				hotWalletAta: systemAtaUsdc.address,
				borrowerAtaAsset: borrowerAtaUsdc.address,
				loanOffer: loanOfferAccount,
				borrower: borrower.publicKey,
				hotWallet: ownerAccountSetting.publicKey,
				tokenProgram: TOKEN_PROGRAM_ID,
			})
			.transaction();
    await sendAndConfirmTransaction(connection, systemUpdateLoanOfferTsx, [
			ownerAccountSetting,
		]);

    // Repay loan offer
    const repayLoanOfferTsx = await program.methods
			.repayLoanOffer(loanOfferId)
			.accounts({
				settingAccount,
				mintAsset: mintUsdcAccount,
        hotWalletAta: hotWalletUsdcAta.address,
				borrower: borrower.publicKey,
        loanAtaAsset: borrowerAtaUsdc.address,
				loanOffer: loanOfferAccount,
				systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
			})
			.transaction();

    await sendAndConfirmTransaction(connection, repayLoanOfferTsx, [
			borrower,
		]);

    // System repay loan offer
    const systemRepayLoanOfferTsx = await program.methods
			.systemRepayLoanOffer(
				loanOfferId,
				new anchor.BN(collateralAmount),
			)
			.accounts({
				system: ownerAccountSetting.publicKey,
				borrower: borrower.publicKey,
				systemAtaAsset: systemAtaUsdc.address,
				mintAsset: mintUsdcAccount,
				loanOffer: loanOfferAccount,
				systemProgram: SystemProgram.programId,
				tokenProgram: TOKEN_PROGRAM_ID,
			})
			.transaction();

    await sendAndConfirmTransaction(connection, systemRepayLoanOfferTsx, [
      ownerAccountSetting,
    ]);
	});
});
