use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Buy Instruction Missing")]
    BuyInstructionMissing,

    #[msg("Invalid Input")]
    InvalidInput,
}
