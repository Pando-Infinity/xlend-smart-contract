import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EnsoLending } from "../target/types/enso_lending";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SOLANA_SCHEMA,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
  getMinimumBalanceForRentExemptMint,
} from "@solana/spl-token";

import { confirm, log } from "./utils";

describe("enso-lending", () => {
  // Set provider, connection and program
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();

  const connection = provider.connection;
  const program = anchor.workspace.EnsoLending as Program<EnsoLending>;

  // Boilerplate
  // Determine dummy token mints and token account address
  const [ownerAccountSetting, hotWallet, usdcMint] = Array.from(
    { length: 3 },
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

      // create USDC token mint
      SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: usdcMint.publicKey,
        lamports,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
      }),
    ];

    await provider
      .sendAndConfirm(tx, [usdcMint])
      .then((sig) => log(connection, sig));
  });

  describe("account setting", () => {
    it("Init Account Setting successfully", async () => {
      const amount = 200;
      const duration = 14;
      const tier_id = "1234_tier_1";

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
        .initSettingAccount(tier_id, amount, new anchor.BN(duration))
        .accounts({
          owner: ownerAccountSetting.publicKey,
          receiver: hotWallet.publicKey,
          settingAccount,
          lendMintAsset: usdcMint.publicKey,
          collateralMintAsset: SOLANA_SCHEMA
          systemProgram: SystemProgram.programId,
        })
        .signers([ownerAccountSetting])
        .rpc()
        .then((sig) => confirm(connection, sig))
        .then((sig) => log(connection, sig));
    });
  });
});
