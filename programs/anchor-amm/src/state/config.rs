use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct Config {

  pub seed: u64,

  // To have the possibility to lock/unlock the pool, since the seed is generic (anyone can find it)
  // To have some immutability at some point and make sure the config doesn't change
  // The 'Option' add 1 byte in front of the Pubkey, InitSpace takes care.
  pub authority: Option<Pubkey>,
  
  pub mint_x: Pubkey,
  pub mint_y: Pubkey,
  pub fee: u16, // To use the pool
  pub locked: bool, // The protocol (pool/amm). For security if something went wrong in it, users can't do anything
  pub lp_bump: u8, // for the LP token mint
  pub config_bump: u8,
}
