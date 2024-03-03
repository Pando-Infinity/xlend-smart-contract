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
  async function checkWalletBalance(tokenAccount: PublicKey): Promise<number> {
    let info = await provider.connection.getAccountInfo(tokenAccount);
    let amount = info.lamports;

    return amount;
  }

  function getAmountDifference(
    beforeAmount: number,
    afterAmount: number
  ): number {
    return afterAmount - beforeAmount;
  }

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

  // Util
  const initSettingAccount = async (params: {
    amount: number;
    duration: number;
    tierId: string;
    lenderFeePercent: number;
    settingAccount: anchor.web3.PublicKey;
  }): Promise<void> => {
    const { amount, duration, lenderFeePercent, tierId, settingAccount } =
      params;
    await program.methods
      .initSettingAccount(
        tierId,
        amount,
        new anchor.BN(duration),
        lenderFeePercent
      )
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
  };

  describe("account setting", () => {
    it("Init Account Setting successfully", async () => {
      const amount = 200;
      const duration = 14;
      const tierId = "1234_tier_1";
      const lenderFeePercent = 0.01;

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

      await initSettingAccount({
        amount,
        duration,
        tierId,
        lenderFeePercent,
        settingAccount,
      });

      // Read data from PDA account
      const {
        amount: fetchedAmount,
        collateralMintAsset,
        lendMintAsset,
        owner,
        receiver,
        tierId: fetchedTierId,
        duration: fetchDuration,
        lenderFeePercent: fetchedLenderFeePercent,
      } = await program.account.settingAccount.fetch(settingAccount);
      assert.equal(fetchedTierId, tierId);
      assert.equal(amount, fetchedAmount);
      assert.equal(fetchedLenderFeePercent, lenderFeePercent);
      assert.equal(duration, fetchDuration.toNumber());
      assert.equal(ownerAccountSetting.publicKey.toString(), owner.toString());
      assert.equal(hotWallet.publicKey.toString(), receiver.toString());
      assert.equal(usdcMint.publicKey.toString(), lendMintAsset.toString());
      assert.equal(
        wrappedSOLTest.publicKey.toString(),
        collateralMintAsset.toString()
      );
    });

    it("Edit Account Setting", async () => {
      const amount = 200;
      const duration = 14;
      const tierId = "1234_tier_1";
      const lenderFeePercent = 0.01;

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

      await initSettingAccount({
        amount,
        duration,
        tierId,
        lenderFeePercent,
        settingAccount,
      });

      const newAmount = 400;
      const newDuration = 28;
      const newLenderFeePercent = 0.02;

      await program.methods
        .editSettingAccount(
          tierId,
          newAmount,
          new anchor.BN(newDuration),
          newLenderFeePercent
        )
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

      const {
        amount: fetchedNewAmount,
        collateralMintAsset: fetchedNewCollateralMintAsset,
        lendMintAsset: fetchedNewLendMintAsset,
        owner: fetchedOwner,
        receiver: fetchedReceiver,
        tierId: fetchedTierId,
        duration: fetchedNewDuration,
        lenderFeePercent: fetchedNewLenderFeePercent,
      } = await program.account.settingAccount.fetch(settingAccount);
      assert.equal(tierId, fetchedTierId);
      assert.equal(newAmount, fetchedNewAmount);
      assert.equal(newLenderFeePercent, fetchedNewLenderFeePercent);
      assert.equal(newDuration, fetchedNewDuration.toNumber());
      assert.equal(
        ownerAccountSetting.publicKey.toString(),
        fetchedOwner.toString()
      );
      assert.equal(hotWallet.publicKey.toString(), fetchedReceiver.toString());
      assert.equal(
        usdcMint.publicKey.toString(),
        fetchedNewLendMintAsset.toString()
      );
      assert.equal(
        wrappedSOLTest.publicKey.toString(),
        fetchedNewCollateralMintAsset.toString()
      );
    });

    it("Close Account Setting", async () => {
      const amount = 200;
      const duration = 14;
      const tierId = "1234_tier_1";
      const lenderFeePercent = 0.01;
      const dataSize = 207; // Replace with the desired account size in bytes
      const expectedLoanRentReturned =
        await program.provider.connection.getMinimumBalanceForRentExemption(
          dataSize
        );

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

      await initSettingAccount({
        amount,
        duration,
        lenderFeePercent,
        settingAccount,
        tierId,
      });

      const walletBalanceBeforeCloseLoan = await checkWalletBalance(
        ownerAccountSetting.publicKey
      );

      await program.methods
        .closeSettingAccount(tierId)
        .accounts({
          owner: ownerAccountSetting.publicKey,
          settingAccount,
          systemProgram: SystemProgram.programId,
        })
        .signers([ownerAccountSetting])
        .rpc({ skipPreflight: true })
        .then((sig) => confirm(connection, sig))
        .then((sig) => log(connection, sig));

      const walletBalanceAfterCloseLoan = await checkWalletBalance(
        ownerAccountSetting.publicKey
      );

      const actualLoanRentReturned = getAmountDifference(
        walletBalanceBeforeCloseLoan,
        walletBalanceAfterCloseLoan
      );

      assert.equal(
        actualLoanRentReturned.toString(),
        expectedLoanRentReturned.toString()
      );

      // Lend offer account closed
      const checkLendOfferAccountInfo =
        await provider.connection.getAccountInfo(settingAccount);
      assert.equal(checkLendOfferAccountInfo, null);
    });
  });
});
