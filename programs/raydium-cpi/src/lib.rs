#![allow(deprecated)]
pub mod constants;
pub mod error;
pub mod instructions;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;

declare_id!("EXZHGtUuunmGujBhM6WxhCbZEqyN9bsAnsvXTyg1rqF");
declare_program!(raydium_launchpad);

#[program]
pub mod raydium_cpi {
    use super::*;

    pub fn create<'info>(
        ctx: Context<'_, '_, '_, 'info, Create<'info>>,
        args: LaunchTokenArgs,
    ) -> Result<()> {
        create_handler(ctx, args)
    }

    pub fn buy<'info>(
        ctx: Context<'_, '_, '_, 'info, Buy<'info>>,
        amount_in: u64,
        minimum_amount_out: u64,
        share_fee_rate: u64,
    ) -> Result<()> {
        buy_handler(ctx, amount_in, minimum_amount_out, share_fee_rate)
    }

    pub fn atomic<'info>(
        ctx: Context<'_, '_, '_, 'info, Atomic<'info>>,
        args: LaunchTokenArgs,
    ) -> Result<()> {
        atmic_handler(ctx, args)
    }
}
