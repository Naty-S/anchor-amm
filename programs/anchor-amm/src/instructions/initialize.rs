
use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token::{ Mint, Token, TokenAccount },
};

use crate::state::Config;


#[derive(Accounts)]
#[instruction(seed: u64)] // for PDA creation
pub struct Initialize<'info> {
  
  #[account(mut)]
  pub initializer: Signer<'info>,
  
  pub mint_x: Account<'info, Mint>,  
  pub mint_y: Account<'info, Mint>,
  
  #[account(
    init,
    payer = initializer,
    seeds = [b"lp", config.key.as_ref()], // need to be unique for each config
    bump,
    mint::decimals = 6,
    mint::authority = config
  )]
  // LP token
  pub mint_lp: Account<'info, Mint>,
  
  #[account(
    init,
    payer = initializer,
    seeds = [b"config", seed.to_le_bytes().as_ref()],
    space = 8 + Config::INIT_SPACE,
    bump
  )]
  // Handle the tokens/vaults
  pub config: Account<'info, Config>,
  
  #[account(
    init,
    payer = initializer,
    associated_token::mint = mint_x,
    associated_token::authority = config,
  )]
  // Pool for x tokens. Users who deposit and swap interact with this only
  pub vault_x: Account<'info, TokenAccount>,
  
  #[account(
    init,
    payer = initializer,
    associated_token::mint = mint_y,
    associated_token::authority = config,
  )]
  // Pool for y tokens. Users who deposit and swap interact with this only
  pub vault_y: Account<'info, TokenAccount>,

  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Initialize<'info> {
  
  pub fn init(
    &mut self,
    seed: u64,
    authority: Option<Pubkey>,
    fee: u16,
    bumps: InitializeBumps
  ) -> Result<()> {

    self.config.set_inner(
      Config {
          seed
        , authority
        , mint_x: self.mint_x.key()
        , mint_y: self.mint_y.key()
        , fee
        , locked: false
        , lp_bump: bumps.mint_lp
        , config_bump: bumps.config
      }
    );

    Ok(())
  }
}
