use anchor_lang::prelude::*;

pub mod seeds {
    pub const CONFIG_SEED: &[u8] = b"global_config";
    pub const POOL_SEED: &[u8] = b"pool";
    pub const POOL_VAULT_SEED: &[u8] = b"pool_vault";
    pub const AUTH_SEED: &[u8] = b"vault_auth_seed";
    pub const EVENT_AUTHORITY: &[u8] = b"__event_authority";
    pub const METADATA_SEED: &[u8] = b"metadata";
}
