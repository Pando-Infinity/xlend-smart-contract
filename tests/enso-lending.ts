import * as anchor from "@coral-xyz/anchor";
import { AnchorError, Program } from "@coral-xyz/anchor";
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
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountIdempotentInstruction,
  createMintToInstruction,
  getOrCreateAssociatedTokenAccount,
  transfer,
  createMint,
} from "@solana/spl-token";

import { confirm, log, getAmountDifference, generateId } from "./utils";
import { assert } from "chai";
import { OPERATE_SYSTEM_SECRET_KEY } from "./accounts/operate-system";

describe("enso-lending", () => {
  async function checkWalletBalance(tokenAccount: PublicKey): Promise<number> {
    let info = await provider.connection.getAccountInfo(tokenAccount);
    let amount = info.lamports;

    return amount;
  }

  // Set provider, connection and program
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  // @ts-ignore
  const providerWallet = provider.wallet.payer as Keypair;

  const connection = provider.connection;
  const program = anchor.workspace.EnsoLending as Program<EnsoLending>;

  // Boilerplate
  // Determine dummy token mints and token account address
  const [lender, hotWallet, usdcMint, wrappedSol] =
    Array.from({ length: 4 }, () => Keypair.generate());
  
  // Create account system to test on local network
  const ownerAccountSetting = Keypair.fromSecretKey(
		Uint8Array.from(OPERATE_SYSTEM_SECRET_KEY)
	);

  const usdcMintDecimal = 6;
  const totalUsdcSupply = 1e9 * 10 ** usdcMintDecimal; // 1000000000 USDC
  const wrappedSolDecimal = 9;

  const providerAtaUsdc = getAssociatedTokenAddressSync(
    usdcMint.publicKey,
    providerWallet.publicKey
  );

  it("Airdrop and create mints", async () => {
    let lamports = await getMinimumBalanceForRentExemptMint(connection);
    let tx = new Transaction();

    tx.instructions = [
      // Airdrop to ownerAccountSetting
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: ownerAccountSetting.publicKey,
        lamports: 0.05 * LAMPORTS_PER_SOL,
      }),

      // Airdrop to lender
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: lender.publicKey,
        lamports: 0.02 * LAMPORTS_PER_SOL,
      }),

			// Airdrop to hot wallet
			SystemProgram.transfer({
				fromPubkey: provider.publicKey,
				toPubkey: hotWallet.publicKey,
				lamports: 0.02 * LAMPORTS_PER_SOL,
			}),

      // create USDC token account
      ...[
        SystemProgram.createAccount({
          fromPubkey: provider.publicKey,
          newAccountPubkey: usdcMint.publicKey,
          lamports,
          space: MINT_SIZE,
          programId: TOKEN_PROGRAM_ID,
        }),

        createInitializeMint2Instruction(
          usdcMint.publicKey,
          usdcMintDecimal,
          provider.publicKey,
          null
        ),

        createAssociatedTokenAccountIdempotentInstruction(
          provider.publicKey,
          providerAtaUsdc,
          provider.publicKey,
          usdcMint.publicKey
        ),

        // mint 1 000 000 000 USDC
        createMintToInstruction(
          usdcMint.publicKey,
          providerAtaUsdc,
          providerWallet.publicKey,
          totalUsdcSupply
        ),
      ],

      // create Wrapped SOL
      ...[
        SystemProgram.createAccount({
          fromPubkey: provider.publicKey,
          newAccountPubkey: wrappedSol.publicKey,
          lamports,
          space: MINT_SIZE,
          programId: TOKEN_PROGRAM_ID,
        }),

        createInitializeMint2Instruction(
          wrappedSol.publicKey,
          wrappedSolDecimal,
          provider.publicKey,
          null
        ),
      ],
    ];

    await provider
      .sendAndConfirm(tx, [usdcMint, wrappedSol, providerWallet])
      .then((sig) => log(connection, sig));

    const providerUsdcBalance = await connection.getTokenAccountBalance(
      providerAtaUsdc
    );
    assert.equal(+providerUsdcBalance.value.amount, totalUsdcSupply);

    const lenderAtaUsdc = await getOrCreateAssociatedTokenAccount(
      connection,
      providerWallet,
      usdcMint.publicKey,
      lender.publicKey
    );

    // transfer 100 USDC to lender
    const usdcTransferToLender = 100 * 10 ** usdcMintDecimal;
    await transfer(
      connection,
      providerWallet,
      providerAtaUsdc,
      lenderAtaUsdc.address,
      providerWallet,
      usdcTransferToLender
    );

    const lenderUsdcBalance = await connection.getTokenAccountBalance(
      lenderAtaUsdc.address
    );
    assert.equal(+lenderUsdcBalance.value.amount, usdcTransferToLender);
  });

  // Util
  const airdrop = async (to: PublicKey ): Promise<void> => {
    let tx = new Transaction();

    tx.instructions = [
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: to,
        lamports: 0.01 * LAMPORTS_PER_SOL,
      }),
    ];

    await provider.sendAndConfirm(tx, []).then((sig) => log(connection, sig));
  };

  const initSettingAccount = async (params: {
		amount: number;
		duration: number;
		tierId: string;
		lenderFeePercent: number;
		borrowerFeePercent: number;
		lendMintAsset: PublicKey;
		collateralMintAsset: PublicKey;
		settingAccount: anchor.web3.PublicKey;
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
				receiver: hotWallet.publicKey,
				settingAccount,
				lendMintAsset,
				collateralMintAsset,
				systemProgram: SystemProgram.programId,
			})
			.signers([ownerAccountSetting])
			.rpc()
			.then((sig) => confirm(connection, sig))
			.then((sig) => log(connection, sig));
	};

  const createLendOffer = async (params: {
    hotWalletAta: PublicKey;
    lender: Keypair;
    lenderAtaAsset: PublicKey;
    lendOffer: PublicKey;
    mintAsset: PublicKey;
    settingAccount: PublicKey;
    offerId: string;
    tierId: string;
    interest: number;
  }): Promise<void> => {
    const {
      hotWalletAta,
      interest,
      lendOffer,
      lender,
      lenderAtaAsset,
      mintAsset,
      offerId,
      settingAccount,
      tierId,
    } = params;

    await program.methods
      .createLendOffer(offerId, tierId, interest)
      .accounts({
        hotWalletAta,
        lender: lender.publicKey,
        lenderAtaAsset,
        lendOffer,
        mintAsset,
        settingAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([lender])
      .rpc()
      .then((sig) => confirm(connection, sig))
      .then((sig) => log(connection, sig));
  };

  const editLendOffer = async (params: {
    offerId: string;
    interest: number;
    lendOffer: PublicKey;
    lender: Keypair;
  }): Promise<void> => {
    const { interest, lendOffer, offerId, lender } = params;

    await program.methods
      .editLendOffer(offerId, interest)
      .accounts({
        lender: lender.publicKey,
        lendOffer,
      })
      .signers([lender])
      .rpc()
      .then((sig) => confirm(connection, sig))
      .then((sig) => log(connection, sig));
  };

  const closeLendOffer = async (params: {
		offerId: string;
		lendOffer: PublicKey;
		lender: Keypair;
	}): Promise<void> => {
    const { offerId, lendOffer, lender } = params;

		await program.methods
			.cancelLendOffer(offerId)
			.accounts({
				lender: lender.publicKey,
        lendOffer,
			})
			.signers([lender])
			.rpc()
			.then((sig) => confirm(connection, sig))
			.then((sig) => log(connection, sig));
	};

  describe("account setting", () => {
    it("Init Account Setting successfully", async () => {
      const amount = 200 * usdcMintDecimal;
      const duration = 14;
      const tierId = "1234_tier_1";
      const lenderFeePercent = 0.01;
      const borrowerFeePercent = 0.01;

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
				lendMintAsset: usdcMint.publicKey,
				collateralMintAsset: wrappedSol.publicKey,
				settingAccount,
				borrowerFeePercent,
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
        borrowerFeePercent: fetchedBorrowerFeePercent,
      } = await program.account.settingAccount.fetch(settingAccount);
      assert.equal(fetchedTierId, tierId);
      assert.equal(amount, fetchedAmount.toNumber());
      assert.equal(fetchedLenderFeePercent, lenderFeePercent);
      assert.equal(fetchedBorrowerFeePercent, borrowerFeePercent);
      assert.equal(duration, fetchDuration.toNumber());
      assert.equal(ownerAccountSetting.publicKey.toString(), owner.toString());
      assert.equal(hotWallet.publicKey.toString(), receiver.toString());
      assert.equal(usdcMint.publicKey.toString(), lendMintAsset.toString());
      assert.equal(
        wrappedSol.publicKey.toString(),
        collateralMintAsset.toString()
      );
    });

    it("Edit Account Setting", async () => {
      const amount = 200 * usdcMintDecimal;
      const duration = 14;
      const tierId = "1234_tier_1";
      const lenderFeePercent = 0.01;
      const borrowerFeePercent = 0.01;

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
				lendMintAsset: usdcMint.publicKey,
				collateralMintAsset: wrappedSol.publicKey,
				settingAccount,
				borrowerFeePercent,
			});

      const newAmount = 400;
      const newDuration = 28;
      const newLenderFeePercent = 0.02;
      const newBorrowerFeePercent = 0.03;

      await program.methods
				.editSettingAccount(
					tierId,
					new anchor.BN(newAmount),
					new anchor.BN(newDuration),
					newLenderFeePercent,
					newBorrowerFeePercent
				)
				.accounts({
					owner: ownerAccountSetting.publicKey,
					receiver: hotWallet.publicKey,
					settingAccount,
					lendMintAsset: usdcMint.publicKey,
					collateralMintAsset: wrappedSol.publicKey,
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
        borrowerFeePercent: fetchedNewBorrowerFeePercent,
      } = await program.account.settingAccount.fetch(settingAccount);
      assert.equal(tierId, fetchedTierId);
      assert.equal(newAmount, fetchedNewAmount.toNumber());
      assert.equal(newLenderFeePercent, fetchedNewLenderFeePercent);
      assert.equal(newBorrowerFeePercent, fetchedNewBorrowerFeePercent);
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
        wrappedSol.publicKey.toString(),
        fetchedNewCollateralMintAsset.toString()
      );
    });

    it("Close Account Setting", async () => {
      const amount = 200 * usdcMintDecimal;
      const duration = 14;
      const tierId = "1234_tier_1";
      const lenderFeePercent = 0.01;
      const borrowerFeePercent = 0.01;
      const dataSize = 215; // Replace with the desired account size in bytes
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
				tierId,
				lenderFeePercent,
				lendMintAsset: usdcMint.publicKey,
				collateralMintAsset: wrappedSol.publicKey,
				settingAccount,
				borrowerFeePercent,
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

      // Setting account closed
      const checkSettingAccountInfo =
        await provider.connection.getAccountInfo(settingAccount);
      assert.equal(checkSettingAccountInfo, null);
    });
  });

  describe("lend offer", () => {
    describe("create lend offer", () => {
      it("create lend offer successfully", async () => {
        const amountTier = 50 * 10 ** usdcMintDecimal;
        const duration = 14;
        const tierId = `tier_id_${generateId(10)}`;
        const lenderFeePercent = 0.01;
        const borrowerFeePercent = 0.01;

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
          amount: amountTier,
          duration,
          tierId,
          lenderFeePercent,
          borrowerFeePercent,
          lendMintAsset: usdcMint.publicKey,
          collateralMintAsset: wrappedSol.publicKey,
          settingAccount,
        });

        const offerId = `lend_offer_id_${generateId(10)}`;
        const interest = 2.1;

        const seedLendOffer = [
          Buffer.from("enso"),
          Buffer.from("lend_offer"),
          lender.publicKey.toBuffer(),
          Buffer.from(offerId),
          program.programId.toBuffer(),
        ];

        const lendOfferAccount = PublicKey.findProgramAddressSync(
          seedLendOffer,
          program.programId
        )[0];

        const hotWalletUsdcAta = await getOrCreateAssociatedTokenAccount(
          connection,
          providerWallet,
          usdcMint.publicKey,
          hotWallet.publicKey
        );

        const lenderAtaUsdc = await getOrCreateAssociatedTokenAccount(
          connection,
          providerWallet,
          usdcMint.publicKey,
          lender.publicKey
        );

        const lenderUsdcBalanceBefore = +(
          await connection.getTokenAccountBalance(lenderAtaUsdc.address)
        ).value.amount;

        await createLendOffer({
          hotWalletAta: hotWalletUsdcAta.address,
          lender,
          lenderAtaAsset: lenderAtaUsdc.address,
          lendOffer: lendOfferAccount,
          mintAsset: usdcMint.publicKey,
          settingAccount,
          interest,
          offerId,
          tierId,
        });

        const lenderUsdcBalanceAfter = +(
          await connection.getTokenAccountBalance(lenderAtaUsdc.address)
        ).value.amount;
        assert.equal(
          +lenderUsdcBalanceAfter,
          lenderUsdcBalanceBefore - amountTier
        );

        const hotWalletUsdcBalance = +(
          await connection.getTokenAccountBalance(hotWalletUsdcAta.address)
        ).value.amount;
        assert.equal(+hotWalletUsdcBalance, amountTier);

        const {
          amount,
          duration: fetchedDuration,
          interest: fetchedInterest,
          lenderFeePercent: fetchedLenderFee,
          lender: fetchedLender,
          lendMintToken,
          offerId: fetchedOfferId,
        } = await program.account.lendOfferAccount.fetch(lendOfferAccount);

        assert.equal(amount.toNumber(), amountTier);
        assert.equal(fetchedDuration.toNumber(), duration);
        assert.equal(fetchedLenderFee, fetchedLenderFee);
        assert.equal(fetchedInterest, interest);
        assert.equal(fetchedLender.toString(), lender.publicKey.toString());
        assert.equal(lendMintToken.toString(), usdcMint.publicKey.toString());
        assert.equal(fetchedOfferId, offerId);
      });

      it("should throw an error if interest is not greater than zero", async () => {
        try {
          const amountTier = 50 * 10 ** usdcMintDecimal;
          const duration = 14;
          const tierId = `tier_id_${generateId(10)}`;
          const lenderFeePercent = 0.01;
          const borrowerFeePercent = 0.01;

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
            amount: amountTier,
            duration,
            tierId,
            lenderFeePercent,
            borrowerFeePercent,
            lendMintAsset: usdcMint.publicKey,
            collateralMintAsset: wrappedSol.publicKey,
            settingAccount,
          });

          const offerId = `lend_offer_id_${generateId(10)}`;
          const interest = 0;

          const seedLendOffer = [
            Buffer.from("enso"),
            Buffer.from("lend_offer"),
            lender.publicKey.toBuffer(),
            Buffer.from(offerId),
            program.programId.toBuffer(),
          ];

          const lendOfferAccount = PublicKey.findProgramAddressSync(
            seedLendOffer,
            program.programId
          )[0];

          const hotWalletUsdcAta = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            hotWallet.publicKey
          );

          const lenderAtaUsdc = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            lender.publicKey
          );

          await createLendOffer({
            hotWalletAta: hotWalletUsdcAta.address,
            lender,
            lenderAtaAsset: lenderAtaUsdc.address,
            lendOffer: lendOfferAccount,
            mintAsset: usdcMint.publicKey,
            settingAccount,
            interest,
            offerId,
            tierId,
          });
        } catch (error) {
          assert.equal(
            error.error.errorMessage,
            "Interest must be greater than 0"
          );
        }
      });

      it("Should throw error if create lend offer account that setting account had not initialized", async () => {
        try {
          const tierId = `tier_id_${generateId(10)}`;

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

          const offerId = `lend_offer_id_${generateId(10)}`;
          const interest = 2.1;

          const seedLendOffer = [
            Buffer.from("enso"),
            Buffer.from("lend_offer"),
            lender.publicKey.toBuffer(),
            Buffer.from(offerId),
            program.programId.toBuffer(),
          ];

          const lendOfferAccount = PublicKey.findProgramAddressSync(
            seedLendOffer,
            program.programId
          )[0];

          const hotWalletUsdcAta = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            hotWallet.publicKey
          );

          const lenderAtaUsdc = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            lender.publicKey
          );

          await createLendOffer({
            hotWalletAta: hotWalletUsdcAta.address,
            lender,
            lenderAtaAsset: lenderAtaUsdc.address,
            lendOffer: lendOfferAccount,
            mintAsset: usdcMint.publicKey,
            settingAccount,
            interest,
            offerId,
            tierId,
          });
        } catch (error) {
          assert.equal(
            error.error.errorMessage,
            "The program expected this account to be already initialized"
          );
        }
      });

      it("Should throw error if lender did not have enough token", async () => {
        try {
          const amountTier = 200 * 10 ** usdcMintDecimal;
          const duration = 14;
          const tierId = `tier_id_${generateId(10)}`;
          const lenderFeePercent = 0.01;
          const borrowerFeePercent = 0.01;

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
            amount: amountTier,
            duration,
            tierId,
            lenderFeePercent,
            borrowerFeePercent,
            lendMintAsset: usdcMint.publicKey,
            collateralMintAsset: wrappedSol.publicKey,
            settingAccount,
          });

          const offerId = `lend_offer_id_${generateId(10)}`;
          const interest = 2.1;

          const seedLendOffer = [
            Buffer.from("enso"),
            Buffer.from("lend_offer"),
            lender.publicKey.toBuffer(),
            Buffer.from(offerId),
            program.programId.toBuffer(),
          ];

          const lendOfferAccount = PublicKey.findProgramAddressSync(
            seedLendOffer,
            program.programId
          )[0];

          const hotWalletUsdcAta = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            hotWallet.publicKey
          );

          const lenderAtaUsdc = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            lender.publicKey
          );

          await createLendOffer({
            hotWalletAta: hotWalletUsdcAta.address,
            lender,
            lenderAtaAsset: lenderAtaUsdc.address,
            lendOffer: lendOfferAccount,
            mintAsset: usdcMint.publicKey,
            settingAccount,
            interest,
            offerId,
            tierId,
          });
        } catch (error) {
          assert.equal(
            error.error.errorMessage,
            "Lender does not have enough assets"
          );
        }
      });

      it('should throw error if lender provide loan mint asset that different lend mint asset in setting account', async () => {
				// create different SPL token
				const newSplToken = await createMint(
					connection,
					providerWallet,
					provider.publicKey,
					provider.publicKey,
					6
				);

				try {
					const tierId = `tier_id_${generateId(10)}`;
					const amountTier = 50 * 10 ** usdcMintDecimal;
					const duration = 14;
					const lenderFeePercent = 0.01;
          const borrowerFeePercent = 0.01;

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

					await initSettingAccount({
						amount: amountTier,
						duration,
						tierId,
						lenderFeePercent,
						lendMintAsset: usdcMint.publicKey,
						collateralMintAsset: wrappedSol.publicKey,
						settingAccount,
            borrowerFeePercent
					});

					const offerId = `lend_offer_id_${generateId(10)}`;
					const interest = 2.1;

					const seedLendOffer = [
						Buffer.from('enso'),
						Buffer.from('lend_offer'),
						lender.publicKey.toBuffer(),
						Buffer.from(offerId),
						program.programId.toBuffer(),
					];

					const lendOfferAccount = PublicKey.findProgramAddressSync(
						seedLendOffer,
						program.programId
					)[0];

					const hotWalletUsdcAta = await getOrCreateAssociatedTokenAccount(
						connection,
						providerWallet,
						usdcMint.publicKey,
						hotWallet.publicKey
					);

					const lenderAtaNewSplToken = await getOrCreateAssociatedTokenAccount(
						connection,
						providerWallet,
						newSplToken,
						lender.publicKey
					);

					await createLendOffer({
						hotWalletAta: hotWalletUsdcAta.address,
						lender,
						lenderAtaAsset: lenderAtaNewSplToken.address,
						lendOffer: lendOfferAccount,
						mintAsset: newSplToken,
						settingAccount,
						interest,
						offerId,
						tierId,
					});
				} catch (error) {
					assert.equal(error.error.errorMessage, 'Invalid mint asset');
				}
			});
    });

    describe("edit lend offer", () => {
      it("Edit lend offer successfully", async () => {
        const amountTier = 10 * 10 ** usdcMintDecimal;
        const duration = 14;
        const tierId = `tier_id_${generateId(10)}`;
        const lenderFeePercent = 0.01;
        const borrowerFeePercent = 0.01;

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
          amount: amountTier,
          duration,
          tierId,
          lenderFeePercent,
          borrowerFeePercent,
          lendMintAsset: usdcMint.publicKey,
          collateralMintAsset: wrappedSol.publicKey,
          settingAccount,
        });

        const offerId = `lend_offer_id_${generateId(10)}`;
        const interest = 2.1;

        const seedLendOffer = [
          Buffer.from("enso"),
          Buffer.from("lend_offer"),
          lender.publicKey.toBuffer(),
          Buffer.from(offerId),
          program.programId.toBuffer(),
        ];

        const lendOfferAccount = PublicKey.findProgramAddressSync(
          seedLendOffer,
          program.programId
        )[0];

        const hotWalletUsdcAta = await getOrCreateAssociatedTokenAccount(
          connection,
          providerWallet,
          usdcMint.publicKey,
          hotWallet.publicKey
        );

        const lenderAtaUsdc = await getOrCreateAssociatedTokenAccount(
          connection,
          providerWallet,
          usdcMint.publicKey,
          lender.publicKey
        );

        await createLendOffer({
          hotWalletAta: hotWalletUsdcAta.address,
          lender,
          lenderAtaAsset: lenderAtaUsdc.address,
          lendOffer: lendOfferAccount,
          mintAsset: usdcMint.publicKey,
          settingAccount,
          interest,
          offerId,
          tierId,
        });

        const newInterest = 5;

        await editLendOffer({
          offerId,
          interest: newInterest,
          lender,
          lendOffer: lendOfferAccount,
        });

        const { interest: fetchedInterest } =
          await program.account.lendOfferAccount.fetch(lendOfferAccount);

        assert.equal(newInterest, fetchedInterest);
      });

      it("Should throw error if lender edit lend offer that not belong to them", async () => {
        const newLender = Keypair.generate();

        // air drop sol to new lender
        await airdrop(newLender.publicKey)

        try {
          const amountTier = 10 * 10 ** usdcMintDecimal;
          const duration = 14;
          const tierId = `tier_id_${generateId(10)}`;
          const lenderFeePercent = 0.01;
          const borrowerFeePercent = 0.01;
  
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
            amount: amountTier,
            duration,
            tierId,
            lenderFeePercent,
            borrowerFeePercent,
            lendMintAsset: usdcMint.publicKey,
            collateralMintAsset: wrappedSol.publicKey,
            settingAccount,
          });
  
          const offerId = `lend_offer_id_${generateId(10)}`;
          const interest = 2.1;
  
          const seedLendOffer = [
            Buffer.from("enso"),
            Buffer.from("lend_offer"),
            lender.publicKey.toBuffer(),
            Buffer.from(offerId),
            program.programId.toBuffer(),
          ];
  
          const lendOfferAccount = PublicKey.findProgramAddressSync(
            seedLendOffer,
            program.programId
          )[0];
  
          const hotWalletUsdcAta = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            hotWallet.publicKey
          );
  
          const lenderAtaUsdc = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            lender.publicKey
          );
  
          await createLendOffer({
            hotWalletAta: hotWalletUsdcAta.address,
            lender,
            lenderAtaAsset: lenderAtaUsdc.address,
            lendOffer: lendOfferAccount,
            mintAsset: usdcMint.publicKey,
            settingAccount,
            interest,
            offerId,
            tierId,
          });
  
          const newInterest = 5;
  
          await editLendOffer({
            offerId,
            interest: newInterest,
            lender: newLender,
            lendOffer: lendOfferAccount,
          });
        } catch (error) {
          assert.equal(error.error.errorMessage, "A seeds constraint was violated")
        }
      });

      it('Should throw an error if lender update interest is not greater than 0', async () => {
        try {
          const amountTier = 10 * 10 ** usdcMintDecimal;
          const duration = 14;
          const tierId = `tier_id_${generateId(10)}`;
          const lenderFeePercent = 0.01;
          const borrowerFeePercent = 0.01;
  
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
						amount: amountTier,
						duration,
						tierId,
						lenderFeePercent,
						lendMintAsset: usdcMint.publicKey,
						collateralMintAsset: wrappedSol.publicKey,
						settingAccount,
						borrowerFeePercent,
					});
  
          const offerId = `lend_offer_id_${generateId(10)}`;
          const interest = 2.1;
  
          const seedLendOffer = [
            Buffer.from("enso"),
            Buffer.from("lend_offer"),
            lender.publicKey.toBuffer(),
            Buffer.from(offerId),
            program.programId.toBuffer(),
          ];
  
          const lendOfferAccount = PublicKey.findProgramAddressSync(
            seedLendOffer,
            program.programId
          )[0];
  
          const hotWalletUsdcAta = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            hotWallet.publicKey
          );
  
          const lenderAtaUsdc = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            lender.publicKey
          );
  
          await createLendOffer({
            hotWalletAta: hotWalletUsdcAta.address,
            lender,
            lenderAtaAsset: lenderAtaUsdc.address,
            lendOffer: lendOfferAccount,
            mintAsset: usdcMint.publicKey,
            settingAccount,
            interest,
            offerId,
            tierId,
          });
  
          const newInterest = -1;
  
          await editLendOffer({
            offerId,
            interest: newInterest,
            lender,
            lendOffer: lendOfferAccount,
          });
        } catch (error) {
          assert.equal(error.error.errorMessage, "Interest must be greater than 0")
        }
      })
    });

    describe('close lend offer', () => {
      it('lender should close the lend offer successfully', async () => {
				const amountTier = 10 * 10 ** usdcMintDecimal;
				const duration = 14;
				const tierId = `tier_id_${generateId(10)}`;
				const lenderFeePercent = 0.01;
        const borrowerFeePercent = 0.01;
        const interest = 2.1;
        const dataSize = 152; // Replace with the desired account size in bytes

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

				await initSettingAccount({
					amount: amountTier,
					duration,
					tierId,
					lenderFeePercent,
					lendMintAsset: usdcMint.publicKey,
					collateralMintAsset: wrappedSol.publicKey,
					settingAccount,
					borrowerFeePercent: borrowerFeePercent,
				});

				const offerId = `lend_offer_id_${generateId(10)}`;

        const seedLendOffer = [
          Buffer.from('enso'),
          Buffer.from('lend_offer'),
          lender.publicKey.toBuffer(),
          Buffer.from(offerId),
          program.programId.toBuffer(),
        ];

        const lendOfferAccount = PublicKey.findProgramAddressSync(
					seedLendOffer,
					program.programId
				)[0];

        const hotWalletUsdcAta = await getOrCreateAssociatedTokenAccount(
					connection,
					providerWallet,
					usdcMint.publicKey,
					hotWallet.publicKey
				);

				const lenderAtaUsdc = await getOrCreateAssociatedTokenAccount(
					connection,
					providerWallet,
					usdcMint.publicKey,
					lender.publicKey
				);

        await createLendOffer({
					hotWalletAta: hotWalletUsdcAta.address,
					lender,
					lenderAtaAsset: lenderAtaUsdc.address,
					lendOffer: lendOfferAccount,
					mintAsset: usdcMint.publicKey,
					settingAccount,
					interest,
					offerId,
					tierId,
				});

        const { status: prevStatus } = await program.account.lendOfferAccount.fetch(
					lendOfferAccount
				);

        assert.equal(prevStatus.hasOwnProperty('created'), true);

				await closeLendOffer({
					lender,
					lendOffer: lendOfferAccount,
					offerId,
				});

        const { status: currentStatus } =
					await program.account.lendOfferAccount.fetch(lendOfferAccount);

        assert.equal(currentStatus.hasOwnProperty('canceled'), true);
			});

      it("Should throw error if lender close lend offer that not belong to them", async () => {
        const newLender = Keypair.generate();

        // air drop sol to new lender
        await airdrop(newLender.publicKey)

        try {
          const amountTier = 10 * 10 ** usdcMintDecimal;
          const duration = 14;
          const tierId = `tier_id_${generateId(10)}`;
          const lenderFeePercent = 0.01;
          const borrowerFeePercent = 0.01;
  
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
            amount: amountTier,
            duration,
            tierId,
            lenderFeePercent,
            lendMintAsset: usdcMint.publicKey,
            collateralMintAsset: wrappedSol.publicKey,
            settingAccount,
            borrowerFeePercent
          });
  
          const offerId = `lend_offer_id_${generateId(10)}`;
          const interest = 2.1;
  
          const seedLendOffer = [
            Buffer.from("enso"),
            Buffer.from("lend_offer"),
            lender.publicKey.toBuffer(),
            Buffer.from(offerId),
            program.programId.toBuffer(),
          ];
  
          const lendOfferAccount = PublicKey.findProgramAddressSync(
            seedLendOffer,
            program.programId
          )[0];
  
          const hotWalletUsdcAta = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            hotWallet.publicKey
          );
  
          const lenderAtaUsdc = await getOrCreateAssociatedTokenAccount(
            connection,
            providerWallet,
            usdcMint.publicKey,
            lender.publicKey
          );
  
          await createLendOffer({
            hotWalletAta: hotWalletUsdcAta.address,
            lender,
            lenderAtaAsset: lenderAtaUsdc.address,
            lendOffer: lendOfferAccount,
            mintAsset: usdcMint.publicKey,
            settingAccount,
            interest,
            offerId,
            tierId,
          });
  
          await closeLendOffer({
						lender,
						lendOffer: lendOfferAccount,
						offerId,
					});
        } catch (error) {
          assert.equal(error.error.errorMessage, "A seeds constraint was violated")
        }
      });
		});
  });
});
