import {
  ComputeBudgetProgram,
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { AnchorProvider } from "@project-serum/anchor";
import {
  OPERATE_SYSTEM_SECRET_KEY,
  HOT_WALLET_SECRET_KEY,
  DEPLOYER_WALLET_SECRET_KEY,
  PROGRAM_ID,
} from "../accounts/staging";

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EnsoLending } from "../target/types/enso_lending";

import enso_lending_idl from "../target/idl/enso_lending.json";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import "dotenv/config";

const enso_lending_idl_string = JSON.stringify(enso_lending_idl);
const enso_lending_idl_obj = JSON.parse(enso_lending_idl_string);

const programId = new PublicKey(PROGRAM_ID);
const connection = new Connection(process.env.RPC_URL as string, "confirmed");

const ownerAccountSetting = Keypair.fromSecretKey(
  Uint8Array.from(OPERATE_SYSTEM_SECRET_KEY)
);
console.log(ownerAccountSetting.publicKey.toBase58());
console.log(ownerAccountSetting.secretKey);
console.log(bs58.encode(ownerAccountSetting.secretKey));

const hotWallet = Keypair.fromSecretKey(Uint8Array.from(HOT_WALLET_SECRET_KEY));
console.log(hotWallet.publicKey.toBase58());
console.log(hotWallet.secretKey);
console.log(bs58.encode(hotWallet.secretKey));

const sol_usd_price_feed_id = "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix";
const usdc_usd_price_feed_id = "5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7";
const mintSolWrappedAccount = new PublicKey(
  "So11111111111111111111111111111111111111112"
);

const mintUsdcAccount = new PublicKey(
  "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
);

const providerWallet = new anchor.Wallet(
  Keypair.fromSecretKey(Uint8Array.from(DEPLOYER_WALLET_SECRET_KEY))
);

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
    hotWallet,
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

const DURATION_TO_SECOND = 1209600; // 14 days

const log = (tx: string) => {
  console.log(
    `https://explorer.solana.com/transaction/${tx}?cluster=custom&customUrl=${connection.rpcEndpoint}`
  );
};

(async () => {
  const amount = 10000000000; // 10000 USDC
  const duration = DURATION_TO_SECOND;
  const tierId = "solana_tier_005";
  const lenderFeePercent = 5;
  const borrowerFeePercent = 5;

  const seedSettingAccount = [
    Buffer.from("enso"),
    Buffer.from("setting_account"),
    Buffer.from(tierId),
    program.programId.toBuffer(),
  ];

  const settingAccount = PublicKey.findProgramAddressSync(
    seedSettingAccount,
    program.programId
  )[0];
  const sol_usd_price_feed = new PublicKey(sol_usd_price_feed_id);
  const usdc_usd_price_feed = new PublicKey(usdc_usd_price_feed_id);

  const initSettingTx = await initSettingAccount({
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
  const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
    units: 1000000,
  });

  const addPriorityFee = ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: 1000,
  });
  const transaction = new Transaction()
    .add(modifyComputeUnits)
    .add(addPriorityFee)
    .add(initSettingTx);
  try {
    const tx = await sendAndConfirmTransaction(connection, transaction, [
      ownerAccountSetting,
    ]);
    console.log(tx);
  } catch (e) {
    console.error(e);
  }
})();
