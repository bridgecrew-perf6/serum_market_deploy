use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use solana_program::{program::invoke, pubkey::Pubkey, system_instruction::create_account};

mod error;
use error::ErrorCode;

mod event;
use event::*;

declare_id!("G5v3swMsJbi2pM6QgGu7rSMKHFd2f5ybn8WuGpMqTdBJ");

#[program]
pub mod serum_market_deploy {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        coin_lot_size: u64,
        pc_lot_size: u64,
        pc_dust_threshold: u64,
    ) -> anchor_lang::Result<()> {
        ctx.accounts.init_account_sizes()?;

        let (vault_signer_pk, vault_signer_nonce) = ctx.accounts.get_vault_signer()?;
        ctx.accounts.initialize_wallets(&vault_signer_pk)?;

        let accs = InitializeMarketAccounts {
            market_state: ctx.accounts.market_state.clone(),
            request_queue: ctx.accounts.request_queue.clone(),
            event_queue: ctx.accounts.event_queue.clone(),
            bids: ctx.accounts.bids.clone(),
            asks: ctx.accounts.asks.clone(),

            coin_wallet: ctx.accounts.coin_wallet.to_account_info().clone(),
            price_wallet: ctx.accounts.price_wallet.to_account_info().clone(),
            coin_mint: ctx.accounts.coin_mint.to_account_info().clone(),
            price_mint: ctx.accounts.price_mint.to_account_info().clone(),
            rent: ctx.accounts.rent.to_account_info().clone(),
        };

        let initialize_ix = serum_dex::instruction::initialize_market(
            &ctx.accounts.market_state.key(),
            &ctx.accounts.serum_dex.key(),
            &ctx.accounts.coin_mint.key(),
            &ctx.accounts.price_mint.key(),
            &ctx.accounts.coin_wallet.key(),
            &ctx.accounts.price_wallet.key(),
            None,
            None,
            None,
            &ctx.accounts.bids.key(),
            &ctx.accounts.asks.key(),
            &ctx.accounts.request_queue.key(),
            &ctx.accounts.event_queue.key(),
            coin_lot_size,
            pc_lot_size,
            vault_signer_nonce,
            pc_dust_threshold,
        )
        .map_err(|_| ErrorCode::InitializeMarketError)?;
        invoke(&initialize_ix, &accs.to_account_infos())?;

        emit!(InitializedEvent {
            coin_mint: ctx.accounts.coin_mint.to_account_info().key(),
            price_mint: ctx.accounts.price_mint.to_account_info().key()
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMarketAccounts<'info> {
    /// CHECK :
    pub market_state: AccountInfo<'info>,
    /// CHECK :
    pub request_queue: AccountInfo<'info>,
    /// CHECK :
    pub event_queue: AccountInfo<'info>,
    /// CHECK :
    pub bids: AccountInfo<'info>,
    /// CHECK :
    pub asks: AccountInfo<'info>,
    /// CHECK :
    pub coin_wallet: AccountInfo<'info>,
    /// CHECK :
    pub price_wallet: AccountInfo<'info>,
    /// CHECK :
    pub coin_mint: AccountInfo<'info>,
    /// CHECK :
    pub price_mint: AccountInfo<'info>,
    /// CHECK :
    pub rent: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        constraint =
        coin_mint.to_account_info().key() != price_mint.to_account_info().key() @ ErrorCode::MatchingMints
    )]
    pub coin_mint: Account<'info, Mint>,
    pub price_mint: Account<'info, Mint>,

    /// CHECK :
    #[account(signer, mut)]
    pub market_state: AccountInfo<'info>,
    /// CHECK :
    #[account(signer, mut)]
    pub request_queue: AccountInfo<'info>,
    /// CHECK :
    #[account(signer, mut)]
    pub event_queue: AccountInfo<'info>,
    /// CHECK :
    #[account(signer, mut)]
    pub bids: AccountInfo<'info>,
    /// CHECK :
    #[account(signer, mut)]
    pub asks: AccountInfo<'info>,
    /// CHECK :
    #[account(mut, signer)]
    pub coin_wallet: AccountInfo<'info>,
    /// CHECK :
    #[account(mut, signer)]
    pub price_wallet: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK :
    pub serum_dex: AccountInfo<'info>,
}

impl<'info> Initialize<'info> {
    const V3_MARKET_STATE_LEN: usize = 12 + std::mem::size_of::<serum_dex::state::MarketState>(); //1476;
    const REQUEST_QUEUE_LEN: usize = 12 + 5120;
    //const EVENT_QUEUE_LEN: usize = 12 + 262144;
    //const BIDS_LEN: usize = 12 + 65536;
    //const ASKS_LEN: usize = 12 + 65536;

    pub fn init_account_sizes(&mut self) -> anchor_lang::Result<()> {
        self.set_size(self.market_state.clone(), Self::V3_MARKET_STATE_LEN)?;
        self.set_size(self.request_queue.clone(), Self::REQUEST_QUEUE_LEN)?;
        //self.set_size(self.event_queue.clone(), Self::EVENT_QUEUE_LEN)?;
        //self.set_size(self.bids.clone(), Self::BIDS_LEN)?;
        //self.set_size(self.asks.clone(), Self::ASKS_LEN)?;

        Ok(())
    }

    fn set_size(
        &mut self,
        account_info: AccountInfo<'info>,
        len: usize,
    ) -> anchor_lang::Result<()> {
        let ix = create_account(
            &self.owner.key(),
            &account_info.key(),
            self.rent.minimum_balance(len),
            len as u64,
            &self.serum_dex.key(),
        );
        invoke(&ix, &[self.owner.to_account_info().clone(), account_info])?;
        Ok(())
    }

    pub fn get_vault_signer(&self) -> anchor_lang::Result<(Pubkey, u64)> {
        for i in 0..=255 {
            let res = serum_dex::state::gen_vault_signer_key(
                i,
                &self.market_state.key(),
                &self.serum_dex.key(),
            );
            match res {
                Ok(pk) => {
                    return Ok((pk, i));
                }
                _ => {}
            }
        }

        Err(ErrorCode::NonceNotFound.into())
    }

    pub fn initialize_wallets(&self, vault_signer: &Pubkey) -> anchor_lang::Result<()> {
        self.initialize_wallet(
            self.price_wallet.to_account_info().clone(),
            self.price_mint.to_account_info().clone(),
            vault_signer,
        )?;

        self.initialize_wallet(
            self.coin_wallet.to_account_info().clone(),
            self.coin_mint.to_account_info().clone(),
            vault_signer,
        )?;

        Ok(())
    }

    fn initialize_wallet(
        &self,
        wallet: AccountInfo<'info>,
        mint: AccountInfo<'info>,
        owner: &Pubkey,
    ) -> anchor_lang::Result<()> {
        let ix = create_account(
            &self.owner.key(),
            &wallet.key(),
            self.rent.minimum_balance(165),
            165,
            &Token::id(),
        );
        invoke(&ix, &[self.owner.to_account_info(), wallet.clone()])?;

        let ix_data = anchor_spl::token::InitializeAccount {
            account: wallet.clone(),
            mint: mint,
            authority: self.owner.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        anchor_spl::token::initialize_account(CpiContext::new(
            self.token_program.to_account_info(),
            ix_data,
        ))?;

        let ix_data = anchor_spl::token::SetAuthority {
            account_or_mint: wallet,
            current_authority: self.owner.to_account_info(),
        };
        anchor_spl::token::set_authority(
            CpiContext::new(self.token_program.to_account_info(), ix_data),
            spl_token::instruction::AuthorityType::AccountOwner,
            Some(*owner),
        )?;

        Ok(())
    }
}
