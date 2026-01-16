use crate::{
    constants::seeds,
    raydium_launchpad::{
        cpi::{accounts::BuyExactIn, buy_exact_in},
        program::RaydiumLaunchpad,
    },
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Buy<'info> {
    pub user: Signer<'info>,

    /// CHECK: checked by cpi
    #[account(
        seeds = [seeds::AUTH_SEED],
        bump,
        seeds::program = raydium_launchpad_program
    )]
    pub authority: AccountInfo<'info>,

    /// CHECK: raydium program checksS
    #[account(
        seeds = [
            seeds::CONFIG_SEED,
            quote_token_mint.key().as_ref(),
            0u8.to_le_bytes().as_ref(),
            0u16.to_le_bytes().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub global_config: AccountInfo<'info>,
    /// CHECK: raydium program checks
    pub platform_config: AccountInfo<'info>,

    /// CHECK: checked by cpi
    #[account(
        mut,
        seeds = [
            seeds::POOL_SEED,
            base_token_mint.key().as_ref(),
            quote_token_mint.key().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub pool_state: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = base_token_mint,
        associated_token::authority = user
    )]
    pub user_base_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = quote_token_mint,
        associated_token::authority = user
    )]
    pub user_quote_token: Account<'info, TokenAccount>,

    /// CHECK: checked by cpi
    #[account(
        mut,
        seeds = [
            seeds::POOL_VAULT_SEED,
            pool_state.key().as_ref(),
            base_token_mint.key().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub base_vault: AccountInfo<'info>,

    /// CHECK: checked by cpi
    #[account(
        mut,
        seeds = [
            seeds::POOL_VAULT_SEED,
            pool_state.key().as_ref(),
            quote_token_mint.key().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub quote_vault: AccountInfo<'info>,

    pub base_token_mint: Account<'info, Mint>,
    pub quote_token_mint: Account<'info, Mint>,

    /// CHECK: checked by cpi
    #[account(
        seeds = [seeds::EVENT_AUTHORITY],
        bump,
        seeds::program = raydium_launchpad_program
    )]
    pub event_authority: AccountInfo<'info>,

    pub raydium_launchpad_program: Program<'info, RaydiumLaunchpad>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn buy_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Buy<'info>>,
    amount_in: u64,
    minimum_amount_out: u64,
    share_fee_rate: u64,
) -> Result<()> {
    let accounts_infos: Vec<AccountInfo> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    let cpi_ctx = CpiContext::new(
        ctx.accounts.raydium_launchpad_program.to_account_info(),
        BuyExactIn {
            payer: ctx.accounts.user.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            global_config: ctx.accounts.global_config.to_account_info(),
            platform_config: ctx.accounts.platform_config.to_account_info(),
            pool_state: ctx.accounts.pool_state.to_account_info(),
            user_base_token: ctx.accounts.user_base_token.to_account_info(),
            user_quote_token: ctx.accounts.user_quote_token.to_account_info(),
            base_vault: ctx.accounts.base_vault.to_account_info(),
            quote_vault: ctx.accounts.quote_vault.to_account_info(),
            base_token_mint: ctx.accounts.base_token_mint.to_account_info(),
            quote_token_mint: ctx.accounts.quote_token_mint.to_account_info(),
            base_token_program: ctx.accounts.token_program.to_account_info(),
            quote_token_program: ctx.accounts.token_program.to_account_info(),
            event_authority: ctx.accounts.event_authority.to_account_info(),
            program: ctx.accounts.raydium_launchpad_program.to_account_info(),
        },
    )
    .with_remaining_accounts(accounts_infos);

    buy_exact_in(cpi_ctx, amount_in, minimum_amount_out, share_fee_rate)?;

    Ok(())
}
