import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
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
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
} from "@solana/spl-token";

import { SmartContract } from "../target/types/smart_contract";
import { assert } from "chai";

describe("enso_lending", () => {
  // Set provider, connection and program
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();

  const connection = provider.connection;
  const program = anchor.workspace.SmartContract as Program<SmartContract>;

  // Boilerplate
  // Determine dummy token mints and token account address
  const [lender, coldWallet, usdcMint] = Array.from({ length: 3 }, () =>
    Keypair.generate()
  );

  const [lenderAtaUSDC, coldWalletAtaUSDC] = [lender, coldWallet].map((a) =>
    getAssociatedTokenAddressSync(usdcMint.publicKey, a.publicKey)
  );

  // Utils
  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };

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

  it("Airdrop and create mints", async () => {
    let lamports = await getMinimumBalanceForRentExemptMint(connection);
    let tx = new Transaction();

    tx.instructions = [
      // Airdrop to lender
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: lender.publicKey,
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

      // Create and initialize amount usdc for lender Ata
      ...[
        createInitializeMint2Instruction(
          usdcMint.publicKey,
          6,
          lender.publicKey,
          null
        ),
        createAssociatedTokenAccountIdempotentInstruction(
          provider.publicKey,
          lenderAtaUSDC,
          lender.publicKey,
          usdcMint.publicKey
        ),
        createMintToInstruction(
          usdcMint.publicKey,
          lenderAtaUSDC,
          lender.publicKey,
          1e9
        ),
      ],

      // Create cold wallet ata
      ...[
        createAssociatedTokenAccountIdempotentInstruction(
          provider.publicKey,
          coldWalletAtaUSDC,
          coldWallet.publicKey,
          usdcMint.publicKey
        ),
      ],
    ];

    await provider.sendAndConfirm(tx, [usdcMint, lender]).then(log);
  });

  describe("lender", () => {
    it("Create Lend order successfully", async () => {
      const amount = 1e6;
      const orderId = "12345abc";
      const interest = 2.1;
      const lenderFee = 2;
      const duration = 14;
      const beforeColdWalletBalance = await connection.getTokenAccountBalance(
        coldWalletAtaUSDC
      );

      const seedLendOrder = [
        Buffer.from("enso"),
        lender.publicKey.toBuffer(),
        Buffer.from(orderId),
      ];

      const lendOrder = PublicKey.findProgramAddressSync(
        seedLendOrder,
        program.programId
      )[0];

      await program.methods
        .createLendOrder(
          orderId,
          new anchor.BN(amount),
          interest,
          new anchor.BN(lenderFee),
          new anchor.BN(duration)
        )
        .accounts({
          lender: lender.publicKey,
          lenderAtaAsset: lenderAtaUSDC,
          cwVault: coldWalletAtaUSDC,
          lendOrder,
          mintAsset: usdcMint.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([lender])
        .rpc()
        .then(confirm)
        .then(log);

      const balanceCw = await connection.getTokenAccountBalance(
        coldWalletAtaUSDC
      );
      assert.equal(
        parseInt(balanceCw.value.amount) -
          parseInt(beforeColdWalletBalance.value.amount),
        amount
      );
    });

    it("Create multiple lend order successfully", async () => {
      const amount = 1e3;
      const orderId = "12345abc";
      const interest = 2.1;
      const lenderFee = 2;
      const duration = 14;
      const beforeColdWalletBalance = await connection.getTokenAccountBalance(
        coldWalletAtaUSDC
      );

      const seedLendOrder = [
        Buffer.from("enso"),
        lender.publicKey.toBuffer(),
        Buffer.from(orderId),
      ];

      const lendOrder = PublicKey.findProgramAddressSync(
        seedLendOrder,
        program.programId
      )[0];

      await program.methods
        .createLendOrder(
          orderId,
          new anchor.BN(amount),
          interest,
          new anchor.BN(lenderFee),
          new anchor.BN(duration)
        )
        .accounts({
          lender: lender.publicKey,
          lenderAtaAsset: lenderAtaUSDC,
          cwVault: coldWalletAtaUSDC,
          lendOrder,
          mintAsset: usdcMint.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .postInstructions([
          await program.methods
            .createLendOrder(
              orderId,
              new anchor.BN(amount),
              interest,
              new anchor.BN(lenderFee),
              new anchor.BN(duration)
            )
            .accounts({
              lender: lender.publicKey,
              lenderAtaAsset: lenderAtaUSDC,
              cwVault: coldWalletAtaUSDC,
              lendOrder,
              mintAsset: usdcMint.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .instruction(),
        ])
        .signers([lender])
        .rpc()
        .then(confirm)
        .then(log);

      const balanceCw = await connection.getTokenAccountBalance(
        coldWalletAtaUSDC
      );
      assert.equal(
        parseInt(balanceCw.value.amount) -
          parseInt(beforeColdWalletBalance.value.amount),
        amount * 2
      );
    });

    it("Should throw an error if Lender is not enough assets", async () => {
      const amount = 1e9;
      const orderId = "12345abc";
      const interest = 2.1;
      const lenderFee = 2;
      const duration = 14;

      const seedLendOrder = [
        Buffer.from("enso"),
        lender.publicKey.toBuffer(),
        Buffer.from(orderId),
      ];

      const lendOrder = PublicKey.findProgramAddressSync(
        seedLendOrder,
        program.programId
      )[0];

      try {
        await program.methods
          .createLendOrder(
            orderId,
            new anchor.BN(amount),
            interest,
            new anchor.BN(lenderFee),
            new anchor.BN(duration)
          )
          .accounts({
            lender: lender.publicKey,
            lenderAtaAsset: lenderAtaUSDC,
            cwVault: coldWalletAtaUSDC,
            lendOrder,
            mintAsset: usdcMint.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([lender])
          .rpc()
          .then(confirm);
      } catch (error) {
        assert.strictEqual(
          error.error.errorMessage,
          "Lender does not have enough assets"
        );
      }
    });

    it('Cancel Lend offer successfully', async () => {
			const orderId = '12345abc';
			const dataSize = 152; // Replace with the desired account size in bytes
			const lendOfferRent =
				await program.provider.connection.getMinimumBalanceForRentExemption(
					dataSize
				);

			const seedLendOrder = [
				Buffer.from('enso'),
				lender.publicKey.toBuffer(),
				Buffer.from(orderId),
			];

			const lendOrder = PublicKey.findProgramAddressSync(
				seedLendOrder,
				program.programId
			)[0];

			const walletBalanceBeforeCloseLoan = await checkWalletBalance(
				lender.publicKey
			);

			await program.methods
				.cancelLendOffer(orderId)
				.accounts({
					lender: lender.publicKey,
					lendOrder,
					tokenProgram: TOKEN_PROGRAM_ID,
					systemProgram: SystemProgram.programId,
				})
				.signers([lender])
				.rpc()
				.then(confirm)
				.then(log);

			const walletBalanceAfterCloseLoan = await checkWalletBalance(
				lender.publicKey
			);

			const actualLoanRentReturned = getAmountDifference(
				walletBalanceBeforeCloseLoan,
				walletBalanceAfterCloseLoan
			);

			const expectedLoanRentReturned = lendOfferRent;

			assert.equal(
				actualLoanRentReturned.toString(),
				expectedLoanRentReturned.toString()
			);

			// Lend offer account closed
			const checkLendOfferAccountInfo =
				await provider.connection.getAccountInfo(lendOrder);
			assert.equal(checkLendOfferAccountInfo, null);
		});

    it('Cancel Lend offer return constraint error', async () => {
			const amount = 1e6;
			const orderId = '123abc';
			const anotherOrderId = '123abc!@#';
			const interest = 2.1;
			const lenderFee = 2;
			const duration = 14;

			const seedLendOrder = [
				Buffer.from('enso'),
				lender.publicKey.toBuffer(),
				Buffer.from(orderId),
			];

			const lendOrder = PublicKey.findProgramAddressSync(
				seedLendOrder,
				program.programId
			)[0];

			await program.methods
				.createLendOrder(
					orderId,
					new anchor.BN(amount),
					interest,
					new anchor.BN(lenderFee),
					new anchor.BN(duration)
				)
				.accounts({
					lender: lender.publicKey,
					lenderAtaAsset: lenderAtaUSDC,
					cwVault: coldWalletAtaUSDC,
					lendOrder,
					mintAsset: usdcMint.publicKey,
					tokenProgram: TOKEN_PROGRAM_ID,
					systemProgram: SystemProgram.programId,
				})
				.signers([lender])
				.rpc()
				.then(confirm)
				.then(log);

      try {
				await program.methods
					.cancelLendOffer(anotherOrderId)
					.accounts({
						lender: lender.publicKey,
						lendOrder,
						tokenProgram: TOKEN_PROGRAM_ID,
						systemProgram: SystemProgram.programId,
					})
					.signers([lender])
					.rpc()
					.then(confirm)
					.then(log);
			} catch (error) {
				assert.strictEqual(
					error.error.errorMessage,
					'A seeds constraint was violated'
				);
			}
		});
  });
});
