use crate::{
    constants::seeds,
    error::ErrorCode,
    raydium_launchpad::{
        cpi::{accounts::InitializeV2, initialize_v2},
        program::RaydiumLaunchpad,
        types::{AmmFeeOn, ConstantCurve, CurveParams, MintParams, VestingParams},
    },
    LaunchTokenArgs,
};
use anchor_lang::{
    prelude::*,
    solana_program::{hash::hash, sysvar},
};
use anchor_spl::{
    metadata::Metadata,
    token::{Mint, Token},
};

#[derive(Accounts)]
pub struct Atomic<'info> {
    pub creator: Signer<'info>,

    /// CHECK: Validated instruction sysvar for checking transaction instructions
    #[account(address = sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,

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

    #[account(
        mut,
        seeds = [
            seeds::METADATA_SEED,
            metadata_program.key().as_ref(),
            base_token_mint.key().as_ref(),
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub metadata_account: SystemAccount<'info>,

    #[account(mut)]
    pub base_token_mint: Signer<'info>,
    pub quote_token_mint: Account<'info, Mint>,

    /// CHECK: checked by cpi
    #[account(
        seeds = [seeds::EVENT_AUTHORITY],
        bump,
        seeds::program = raydium_launchpad_program
    )]
    pub event_authority: AccountInfo<'info>,

    pub rent_program: Sysvar<'info, Rent>,
    pub metadata_program: Program<'info, Metadata>,
    pub raydium_launchpad_program: Program<'info, RaydiumLaunchpad>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

fn instruction_discriminator(name: &str) -> [u8; 8] {
    let mut discriminator = [0u8; 8];
    let hash = hash(format!("global:{}", name).as_bytes());
    discriminator.copy_from_slice(&hash.to_bytes()[..8]);
    discriminator
}

pub fn atmic_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Atomic<'info>>,
    args: LaunchTokenArgs,
) -> Result<()> {
    let current_idx =
        sysvar::instructions::load_current_index_checked(&ctx.accounts.instructions_sysvar)?;

    let buy_token_disc = instruction_discriminator("buy");

    let mut found_buy_token = false;
    let mut instruction_index = current_idx + 1;

    loop {
        match sysvar::instructions::load_instruction_at_checked(
            instruction_index as usize,
            &ctx.accounts.instructions_sysvar,
        ) {
            Ok(instruction) => {
                if instruction.program_id == crate::id()
                    && instruction.data.len() >= 8
                    && instruction.data[0..8] == buy_token_disc
                {
                    found_buy_token = true;
                    break;
                }
                instruction_index += 1;
            }
            Err(_) => break,
        }
    }

    require!(found_buy_token, ErrorCode::BuyInstructionMissing);

    let cpi_ctx = CpiContext::new(
        ctx.accounts.raydium_launchpad_program.to_account_info(),
        InitializeV2 {
            payer: ctx.accounts.creator.to_account_info(),
            creator: ctx.accounts.creator.to_account_info(),
            global_config: ctx.accounts.global_config.to_account_info(),
            platform_config: ctx.accounts.platform_config.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            pool_state: ctx.accounts.pool_state.to_account_info(),
            base_mint: ctx.accounts.base_token_mint.to_account_info(),
            quote_mint: ctx.accounts.quote_token_mint.to_account_info(),
            base_vault: ctx.accounts.base_vault.to_account_info(),
            quote_vault: ctx.accounts.quote_vault.to_account_info(),
            metadata_account: ctx.accounts.metadata_account.to_account_info(),
            base_token_program: ctx.accounts.token_program.to_account_info(),
            quote_token_program: ctx.accounts.token_program.to_account_info(),
            metadata_program: ctx.accounts.metadata_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent_program: ctx.accounts.rent_program.to_account_info(),
            event_authority: ctx.accounts.event_authority.to_account_info(),
            program: ctx.accounts.raydium_launchpad_program.to_account_info(),
        },
    );

    let params = MintParams {
        name: args.name.clone(),
        symbol: args.symbol.clone(),
        uri: args.uri.clone(),
        decimals: 6,
    };

    let curve_params = CurveParams::Constant {
        data: ConstantCurve {
            supply: 1000000000000000,
            total_base_sell: 793100000000000,
            total_quote_fund_raising: 30000000000,
            migrate_type: 1,
        },
    };

    let vesting_params = VestingParams {
        total_locked_amount: 0,
        cliff_period: 0,
        unlock_period: 0,
    };

    let amm_fee_on = AmmFeeOn::QuoteToken;

    initialize_v2(cpi_ctx, params, curve_params, vesting_params, amm_fee_on)?;

    Ok(())
}
