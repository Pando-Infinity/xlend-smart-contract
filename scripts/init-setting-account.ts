import { Connection, Keypair, PublicKey, SystemProgram, clusterApiUrl } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EnsoLending } from "../target/types/enso_lending";
import { confirm, log } from "../tests/utils";

const program = anchor.workspace.EnsoLending as Program<EnsoLending>;
const connection = new Connection(clusterApiUrl('devnet'), 'confirmed')

const sol_usd_price_feed_id = 'J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix'
const usdc_usd_price_feed_id = '5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7'

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
  
}): Promise<void> => {
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
  await program.methods
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
    .signers([ownerAccountSetting])
    .rpc()
    .then((sig) => confirm(connection, sig))
    .then((sig) => log(connection, sig));
};

const main = async () => {
  const lendPriceFeedAccount = new PublicKey(usdc_usd_price_feed_id)
  const collateralPriceFeedAccount = new PublicKey(sol_usd_price_feed_id)

};

main();
