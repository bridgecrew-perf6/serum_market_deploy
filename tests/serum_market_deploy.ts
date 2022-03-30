import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SerumMarketDeploy } from "../target/types/serum_market_deploy";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { sleep } from "@project-serum/common";

describe("serum_market_deploy", () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.SerumMarketDeploy as Program<SerumMarketDeploy>;

  let coinMint : Token = null;
  let priceMint : Token = null;

  let owner = Keypair.generate();

  let marketState = Keypair.generate();
  let requestQueue = Keypair.generate();
  let eventQueue = Keypair.generate();
  let bids = Keypair.generate();
  let asks = Keypair.generate();

  let coinWallet = Keypair.generate();
  let priceWallet = Keypair.generate();

  const serumPK = new PublicKey("DESVgJVGajEgKGXhb6XmqDHGz3VjdgP7rEVESBgxmroY");

  it("Is initialized!", async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(owner.publicKey, 2_000_000_000),
      "confirmed"
    );

    console.log("AIRDROPPPED 2 SOL");
    await sleep(10000);

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(owner.publicKey, 2_000_000_000),
      "confirmed"
    );

    console.log("AIRDROPPPED 2 SOL");
    await sleep(10000);

    coinMint = await Token.createMint(
      provider.connection,
      owner,
      owner.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    priceMint = await Token.createMint(
      provider.connection,
      owner,
      owner.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    let listener = program.addEventListener("InitializedEvent", (event, slot) => {
      console.log("Price mint Pubkey : ", event.priceMint);
      console.log("Coin mint Pubkey : ", event.coinMint);
    });

    let tx_prep = new anchor.web3.Transaction();
    tx_prep.add(SystemProgram.createAccount({
      fromPubkey : owner.publicKey,
      newAccountPubkey : eventQueue.publicKey,
      lamports : await provider.connection.getMinimumBalanceForRentExemption(262144 + 12),
      space : 262144 + 12,
      programId : serumPK
    }));
    tx_prep.add(SystemProgram.createAccount({
      fromPubkey : owner.publicKey,
      newAccountPubkey : bids.publicKey,
      lamports : await provider.connection.getMinimumBalanceForRentExemption(65536 + 12),
      space : 65536 + 12,
      programId : serumPK
    }));
    tx_prep.add(SystemProgram.createAccount({
      fromPubkey : owner.publicKey,
      newAccountPubkey : asks.publicKey,
      lamports : await provider.connection.getMinimumBalanceForRentExemption(65536 + 12),
      space : 65536 + 12,
      programId : serumPK
    }));

    await anchor.web3.sendAndConfirmTransaction(
      provider.connection, 
      tx_prep, 
      [owner, eventQueue, bids, asks]
    );

    console.log("ACCOUNTS ARE PREPARED");

    await program.rpc.initialize(
      new anchor.BN(10),
      new anchor.BN(10),
      new anchor.BN(1),
      {
      accounts : {
        owner : owner.publicKey,
        coinMint : coinMint.publicKey,
        priceMint : priceMint.publicKey,
        marketState : marketState.publicKey,
        requestQueue : requestQueue.publicKey,
        eventQueue : eventQueue.publicKey,
        bids : bids.publicKey,
        asks : asks.publicKey,
        coinWallet : coinWallet.publicKey,
        priceWallet : priceWallet.publicKey,

        systemProgram : SystemProgram.programId,
        tokenProgram : TOKEN_PROGRAM_ID,
        rent : anchor.web3.SYSVAR_RENT_PUBKEY,

        serumDex : serumPK
      },
      signers : [
        owner, 
        marketState, 
        requestQueue, 
        eventQueue, 
        bids, 
        asks, 
        coinWallet, 
        priceWallet
      ]
    });
  });
});
