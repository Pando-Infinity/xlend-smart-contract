import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EnsoLending } from "../target/types/enso_lending";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
  getMinimumBalanceForRentExemptMint,
  createInitializeMint2Instruction,
} from "@solana/spl-token";

import { confirm, log } from "./utils";
import { assert } from "chai";

describe("enso-lending", () => {
  // Set provider, connection and program
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();

  const connection = provider.connection;
  const program = anchor.workspace.EnsoLending as Program<EnsoLending>;

  // Boilerplate
  // Determine dummy token mints and token account address
  const [ownerAccountSetting, hotWallet, usdcMint, wrappedSOLTest] = Array.from(
    { length: 4 },
    () => Keypair.generate()
  );

  it("Airdrop and create mints", async () => {
    let lamports = await getMinimumBalanceForRentExemptMint(connection);
    let tx = new Transaction();

    tx.instructions = [
      // Airdrop to ownerAccountSetting
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: ownerAccountSetting.publicKey,
        lamports: 0.01 * LAMPORTS_PER_SOL,
      }),

      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: hotWallet.publicKey,
        lamports: 0.01 * LAMPORTS_PER_SOL,
      }),

      // create USDC token mint
      SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: usdcMint.publicKey,
        lamports,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
      }),

      createInitializeMint2Instruction(
        usdcMint.publicKey,
        6,
        provider.publicKey,
        null
      ),

      // create Wrapped SOL
      SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: wrappedSOLTest.publicKey,
        lamports,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
      }),

      createInitializeMint2Instruction(
        wrappedSOLTest.publicKey,
        6,
        provider.publicKey,
        null
      ),
    ];

    await provider
      .sendAndConfirm(tx, [usdcMint, wrappedSOLTest])
      .then((sig) => log(connection, sig));
  });

  describe("account setting", () => {
    it("Init Account Setting successfully", async () => {
      const amount = 200;
      const duration = 14;
      const tier_id = "1234_tier_1";
      const lender_fee_percent = 0.01

      const seedSettingAccount = [
        Buffer.from("enso"),
        Buffer.from(tier_id),
        program.programId.toBuffer(),
        ownerAccountSetting.publicKey.toBuffer(),
      ];

      const settingAccount = PublicKey.findProgramAddressSync(
        seedSettingAccount,
        program.programId
      )[0];

      await program.methods
        .initSettingAccount(tier_id, amount, new anchor.BN(duration), lender_fee_percent)
        .accounts({
          owner: ownerAccountSetting.publicKey,
          receiver: hotWallet.publicKey,
          settingAccount,
          lendMintAsset: usdcMint.publicKey,
          collateralMintAsset: wrappedSOLTest.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([ownerAccountSetting])
        .rpc({ skipPreflight: true })
        .then((sig) => confirm(connection, sig))
        .then((sig) => log(connection, sig));

      // Read data from PDA account
      const {
        amount: fetchAmount,
        collateralMintAsset,
        lendMintAsset,
        owner,
        receiver,
        tierId,
        duration: fetchDuration,
        lenderFeePercent
      } = await program.account.settingAccount.fetch(settingAccount);
      assert.equal(tier_id, tierId);
      assert.equal(amount, fetchAmount);
      assert.equal(lender_fee_percent, lenderFeePercent);
      assert.equal(duration, fetchDuration.toNumber());
      assert.equal(ownerAccountSetting.publicKey.toString(), owner.toString());
      assert.equal(hotWallet.publicKey.toString(), receiver.toString());
      assert.equal(usdcMint.publicKey.toString(), lendMintAsset.toString());
      assert.equal(
        wrappedSOLTest.publicKey.toString(),
        collateralMintAsset.toString()
      );
    });
  });
});
