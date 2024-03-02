import * as anchor from "@coral-xyz/anchor";

export const confirm = async (
  connection: anchor.web3.Connection,
  signature: string
): Promise<string> => {
  const block = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    signature,
    ...block,
  });
  return signature;
};

export const log = async (
  connection: anchor.web3.Connection,
  signature: string
): Promise<string> => {
  console.log(
    `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
  );
  return signature;
};
