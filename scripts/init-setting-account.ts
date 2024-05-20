import { Connection, Keypair, PublicKey, SystemProgram, clusterApiUrl, sendAndConfirmTransaction } from "@solana/web3.js";
import { AnchorProvider } from '@project-serum/anchor';
import {
	OPERATE_SYSTEM_SECRET_KEY,
	HOT_WALLET_SECRET_KEY,
} from '../tests/accounts';

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EnsoLending } from "../target/types/enso_lending";
import { confirm, log } from "../tests/utils";

import enso_lending_idl from '../target/idl/enso_lending.json';

const enso_lending_idl_string = JSON.stringify(enso_lending_idl);
const enso_lending_idl_obj = JSON.parse(enso_lending_idl_string);
const PROGRAM_ID_DEV_NET = 'BderhzujHHQNjhCZGRjLWnN2XQ66q4EZnZx2p5WLJnBV';

const programId = new PublicKey(PROGRAM_ID_DEV_NET);
const connection = new Connection(clusterApiUrl('devnet'), 'confirmed')

const ownerAccountSetting = Keypair.fromSecretKey(
	Uint8Array.from(OPERATE_SYSTEM_SECRET_KEY)
);
const hotWallet = Keypair.fromSecretKey(Uint8Array.from(HOT_WALLET_SECRET_KEY));

const usdcMintDecimal = 6;
const sol_usd_price_feed_id = 'J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix'
const usdc_usd_price_feed_id = '5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7'
const mintSolWrappedAccount = new PublicKey(
  'So11111111111111111111111111111111111111112'
);

const mintUsdcAccount = new PublicKey(
	'Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr'
);

const providerWallet = new anchor.Wallet(Keypair.generate());

console.log(`Provider Wallet: ${providerWallet.publicKey.toBase58()}`);

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

const initSettingAccount = async (params: {
  amount: number;
  duration: number;
  tierId: string;
  lenderFeePercent: number;
  borrowerFeePercent: number;
  lendMintAsset: PublicKey;
  collateralMintAsset: PublicKey;
  settingAccount: anchor.web3.PublicKey;
  collateralPriceFeedAccount: PublicKey;
  lendPriceFeedAccount: PublicKey;
  ownerAccountSetting: Keypair;
  hotWallet: PublicKey;
  
}) => {
  const {
    amount,
    duration,
    lenderFeePercent,
    borrowerFeePercent,
    tierId,
    lendMintAsset,
    collateralMintAsset,
    settingAccount,
    collateralPriceFeedAccount,
    lendPriceFeedAccount,
    ownerAccountSetting,
    hotWallet
  } = params;
  return await program.methods
    .initSettingAccount(
      tierId,
      new anchor.BN(amount),
      new anchor.BN(duration),
      lenderFeePercent,
      borrowerFeePercent
    )
    .accounts({
      owner: ownerAccountSetting.publicKey,
      receiver: hotWallet,
      settingAccount,
      lendMintAsset,
      collateralMintAsset,
      systemProgram: SystemProgram.programId,
      collateralPriceFeedAccount,
      lendPriceFeedAccount,
    })
    .transaction();
};

(async () => {
	const amount = 100000000;
	const duration = 1209600;
	const tierId = 'tier_001';
	const lenderFeePercent = 0;
	const borrowerFeePercent = 0;

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
	const sol_usd_price_feed = new PublicKey(sol_usd_price_feed_id);
	const usdc_usd_price_feed = new PublicKey(usdc_usd_price_feed_id);

	const transaction = await initSettingAccount({
		amount,
		duration,
		tierId,
		lenderFeePercent,
		lendMintAsset: mintUsdcAccount,
		collateralMintAsset: mintSolWrappedAccount,
		settingAccount,
		borrowerFeePercent,
		lendPriceFeedAccount: usdc_usd_price_feed,
		collateralPriceFeedAccount: sol_usd_price_feed,
		ownerAccountSetting: ownerAccountSetting,
		hotWallet: hotWallet.publicKey,
	});

  await sendAndConfirmTransaction(connection, transaction, [
		ownerAccountSetting,
  ]).then((tx) => {
    console.log(tx);
  });
})();

(async () => {
	const amount = 200000000;
	const duration = 1209600;
	const tierId = 'tier_002';
	const lenderFeePercent = 0;
	const borrowerFeePercent = 0;

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
	const sol_usd_price_feed = new PublicKey(sol_usd_price_feed_id);
	const usdc_usd_price_feed = new PublicKey(usdc_usd_price_feed_id);

	const transaction = await initSettingAccount({
		amount,
		duration,
		tierId,
		lenderFeePercent,
		lendMintAsset: mintUsdcAccount,
		collateralMintAsset: mintSolWrappedAccount,
		settingAccount,
		borrowerFeePercent,
		lendPriceFeedAccount: usdc_usd_price_feed,
		collateralPriceFeedAccount: sol_usd_price_feed,
		ownerAccountSetting: ownerAccountSetting,
		hotWallet: hotWallet.publicKey,
	});

	await sendAndConfirmTransaction(connection, transaction, [
		ownerAccountSetting,
	]).then((tx) => {
		console.log(tx);
	});
})();

(async () => {
	const amount = 500000000;
	const duration = 1209600;
	const tierId = 'tier_003';
	const lenderFeePercent = 0;
	const borrowerFeePercent = 0;

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
	const sol_usd_price_feed = new PublicKey(sol_usd_price_feed_id);
	const usdc_usd_price_feed = new PublicKey(usdc_usd_price_feed_id);

	const transaction = await initSettingAccount({
		amount,
		duration,
		tierId,
		lenderFeePercent,
		lendMintAsset: mintUsdcAccount,
		collateralMintAsset: mintSolWrappedAccount,
		settingAccount,
		borrowerFeePercent,
		lendPriceFeedAccount: usdc_usd_price_feed,
		collateralPriceFeedAccount: sol_usd_price_feed,
		ownerAccountSetting: ownerAccountSetting,
		hotWallet: hotWallet.publicKey,
	});

	await sendAndConfirmTransaction(connection, transaction, [
		ownerAccountSetting,
	]).then((tx) => {
		console.log(tx);
	});
})();

(async () => {
	const amount = 1000000000;
	const duration = 1209600;
	const tierId = 'tier_004';
	const lenderFeePercent = 0;
	const borrowerFeePercent = 0;

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
	const sol_usd_price_feed = new PublicKey(sol_usd_price_feed_id);
	const usdc_usd_price_feed = new PublicKey(usdc_usd_price_feed_id);

	const transaction = await initSettingAccount({
		amount,
		duration,
		tierId,
		lenderFeePercent,
		lendMintAsset: mintUsdcAccount,
		collateralMintAsset: mintSolWrappedAccount,
		settingAccount,
		borrowerFeePercent,
		lendPriceFeedAccount: usdc_usd_price_feed,
		collateralPriceFeedAccount: sol_usd_price_feed,
		ownerAccountSetting: ownerAccountSetting,
		hotWallet: hotWallet.publicKey,
	});

	await sendAndConfirmTransaction(connection, transaction, [
		ownerAccountSetting,
	]).then((tx) => {
		console.log(tx);
	});
})();


(async () => {
	const amount = 2000000000;
	const duration = 1209600;
	const tierId = 'tier_005';
	const lenderFeePercent = 0;
	const borrowerFeePercent = 0;

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
	const sol_usd_price_feed = new PublicKey(sol_usd_price_feed_id);
	const usdc_usd_price_feed = new PublicKey(usdc_usd_price_feed_id);

	const transaction = await initSettingAccount({
		amount,
		duration,
		tierId,
		lenderFeePercent,
		lendMintAsset: mintUsdcAccount,
		collateralMintAsset: mintSolWrappedAccount,
		settingAccount,
		borrowerFeePercent,
		lendPriceFeedAccount: usdc_usd_price_feed,
		collateralPriceFeedAccount: sol_usd_price_feed,
		ownerAccountSetting: ownerAccountSetting,
		hotWallet: hotWallet.publicKey,
	});

	await sendAndConfirmTransaction(connection, transaction, [
		ownerAccountSetting,
	]).then((tx) => {
		console.log(tx);
	});
})();

(async () => {
	const amount = 5000000000;
	const duration = 1209600;
	const tierId = 'tier_006';
	const lenderFeePercent = 0;
	const borrowerFeePercent = 0;

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
	const sol_usd_price_feed = new PublicKey(sol_usd_price_feed_id);
	const usdc_usd_price_feed = new PublicKey(usdc_usd_price_feed_id);

	const transaction = await initSettingAccount({
		amount,
		duration,
		tierId,
		lenderFeePercent,
		lendMintAsset: mintUsdcAccount,
		collateralMintAsset: mintSolWrappedAccount,
		settingAccount,
		borrowerFeePercent,
		lendPriceFeedAccount: usdc_usd_price_feed,
		collateralPriceFeedAccount: sol_usd_price_feed,
		ownerAccountSetting: ownerAccountSetting,
		hotWallet: hotWallet.publicKey,
	});

	await sendAndConfirmTransaction(connection, transaction, [
		ownerAccountSetting,
	]).then((tx) => {
		console.log(tx);
	});
})();