#![allow(unexpected_cfgs, deprecated)]
use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

pub use instructions::*;
pub use state::*;

declare_id!("FN18FMQY7iizFvpQc2in2rwrwQRwshV4FcvnkaxDo1Wv");

#[program]
pub mod anchor_amm {
    use super::*;

    pub fn initialize(
          ctx: Context<Initialize>
        , seed: u64
        , authority: Option<Pubkey>
        , fee: u16
    ) -> Result<()> {
        ctx.accounts.init(seed, authority, fee, ctx.bumps)
    }
}
