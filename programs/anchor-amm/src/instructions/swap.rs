use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token::{ Mint, Token, TokenAccount, Transfer, transfer },
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{error::AmmError, state::Config};

#[derive(Accounts)]
pub struct Swap<'info> {

  #[account(mut)]
  pub swaper: Signer<'info>,

  pub mint_x: Account<'info, Mint>,  
  pub mint_y: Account<'info, Mint>,
  
  #[account(
    has_one = mint_x,
    has_one = mint_y,
    seeds = [b"config", config.seed.to_le_bytes().as_ref()],
    bump = config.config_bump
  )]
  // Handle the tokens/vaults
  pub config: Account<'info, Config>,

  #[account(
    mut,
    associated_token::mint = mint_x,
    associated_token::authority = config,
  )]
  // Pool for x tokens. Users who deposit and swap interact with this only
  pub vault_x: Account<'info, TokenAccount>,
  
  #[account(
    mut,
    associated_token::mint = mint_y,
    associated_token::authority = config,
  )]
  // Pool for y tokens. Users who deposit and swap interact with this only
  pub vault_y: Account<'info, TokenAccount>,

  #[account(
    init_if_needed,
    payer = swaper,
    associated_token::mint = mint_x,
    associated_token::authority = swaper,
  )]
  // 
  pub swaper_ata_x: Account<'info, TokenAccount>,
  
  #[account(
    init_if_needed,
    payer = swaper,
    associated_token::mint = mint_y,
    associated_token::authority = swaper,
  )]
  // 
  pub swaper_ata_y: Account<'info, TokenAccount>,

  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Swap<'info> {
  
  // 
  pub fn swap(
    &mut self,
    is_x: bool, // Which token to swap from
    amount: u64, // How much to swap from that token
    min: u64 // Min amount of tokens user wants to receive. Slipage protection
  ) -> Result<()> {

    require!(self.config.locked == false, AmmError::PoolLocked);
    require!(amount != 0, AmmError::InvalidAmount);

    // Init curve
    // Has the amount of tokens in the pool
    // Has the % of the tokens to have to trade for the other one
    let mut curve = ConstantProduct::init(
      self.vault_x.amount,
      self.vault_y.amount,
      self.vault_x.amount,
      self.config.fee,
      None
    ).map_err(AmmError::from)?;

    // Select which LPool token pair from constant product
    let lp = match is_x {
      true => LiquidityPair::X,
      false => LiquidityPair::Y,
    };

    let res = curve.swap(lp, amount, min).map_err(AmmError::from)?;

    require!(res.deposit != 0, AmmError::InvalidAmount);
    require!(res.withdraw != 0, AmmError::InvalidAmount);

    // Swap tokens
    self.deposit_token(is_x, res.deposit)?;
    self.withdraw_token(is_x, res.withdraw)?;

    // Send back fees to LProvider
    

    Ok(())
  }

  // Deposit 'x' or 'y' tokens to the corresponding vault
  fn deposit_token(
    &self,
    is_x: bool,
    amount: u64,
  ) -> Result<()> {

    let (from, to) = match is_x {
      true => (self.swaper_ata_x.to_account_info(), self.vault_x.to_account_info()),
      false => (self.swaper_ata_y.to_account_info(), self.vault_y.to_account_info())
    };

    let cpi_program = self.token_program.to_account_info();
    let cpi_accs = Transfer{from, to, authority: self.swaper.to_account_info() };
    let ctx = CpiContext::new(cpi_program, cpi_accs);

    transfer(ctx, amount)
  }

  // Receive the token wanted
  fn withdraw_token(
    &self,
    is_x: bool,
    amount: u64,
  ) -> Result<()> {

    let (from, to) = match is_x {
      true => (self.vault_y.to_account_info(), self.swaper_ata_y.to_account_info()),
      false => (self.vault_x.to_account_info(), self.swaper_ata_x.to_account_info())
    };

    // Because transfer from vault
    let seeds: [&[&[u8]]; 1] = [&[
      b"config",
      &self.config.seed.to_le_bytes()[..],
      &[self.config.config_bump]
    ]];

    let cpi_program = self.token_program.to_account_info();
    let cpi_accs = Transfer{from, to, authority: self.swaper.to_account_info() };
    let ctx = CpiContext::new_with_signer(
      cpi_program,
      cpi_accs,
      &seeds
    );

    transfer(ctx, amount)
  }
}
