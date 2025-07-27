use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token::{ Mint, Token, TokenAccount, Transfer, Burn, transfer, burn },
};
use constant_product_curve::ConstantProduct;

use crate::{error::AmmError, state::Config};


#[derive(Accounts)]
pub struct Withdraw<'info> {

  #[account(mut)]
  pub withdrawer: Signer<'info>,
  
  pub mint_x: Account<'info, Mint>,  
  pub mint_y: Account<'info, Mint>,
  
  #[account(
    mut,
    seeds = [b"lp", config.key().as_ref()],
    bump = config.lp_bump
  )]
  // LP token
  pub mint_lp: Account<'info, Mint>,
  
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
  // Pool for x tokens. Users who withdraw and swap interact with this only
  pub vault_x: Account<'info, TokenAccount>,
  
  #[account(
    mut,
    associated_token::mint = mint_y,
    associated_token::authority = config,
  )]
  // Pool for y tokens. Users who withdraw and swap interact with this only
  pub vault_y: Account<'info, TokenAccount>,

  #[account(
    mut,
    associated_token::mint = mint_x,
    associated_token::authority = withdrawer,
  )]
  // To where to withdraw x tokens from its vault
  pub withdrawer_ata_x: Account<'info, TokenAccount>,
  
  #[account(
    mut,
    associated_token::mint = mint_y,
    associated_token::authority = withdrawer,
  )]
  // To where to withdraw y tokens from its vault
  pub withdrawer_ata_y: Account<'info, TokenAccount>,

  #[account(
    mut,
    associated_token::mint = mint_lp,
    associated_token::authority = withdrawer,
  )]
  // To where burn the lp tokens
  pub withdrawer_ata_lp: Account<'info, TokenAccount>,
  
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Withdraw<'info> {
  
  // Get back the token pair deposited in the pool
  pub fn withdraw(
    &mut self,
    lp_amount: u64, // how many lp tokens withdrawer wants to burn
    
    // Min tokens withdrawer wants to receive back. To avoid slipage
    min_x: u64,
    min_y: u64,
  ) -> Result<()> {

    require!(self.config.locked == false, AmmError::PoolLocked);
    require!(lp_amount != 0, AmmError::InvalidAmount);
    require!(min_x != 0 || min_y != 0, AmmError::InvalidAmount);

    // Calc how many 'x' and 'y' tokens user has to get
    let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
      // current state of the pool
        self.vault_x.amount
      , self.vault_y.amount
      , self.mint_lp.supply
      , lp_amount
      , 6 // from lp decimanls
    ).map_err(AmmError::from).unwrap();

    require!(min_x <= amounts.x && min_y <= amounts.y, AmmError::SlippageExceeded);

    self.withdraw_token(true, amounts.x)?;
    self.withdraw_token(false, amounts.y)?;
    self.burn_lp_tokens(lp_amount)
  }

  // Withdraw 'x' or 'y' tokens to the corresponding vault
  fn withdraw_token(
    &self,
    is_x: bool,
    amount: u64,
  ) -> Result<()> {

    let (from, to) = match is_x {
      true => (self.vault_x.to_account_info(), self.withdrawer_ata_x.to_account_info()),
      false => (self.vault_y.to_account_info(), self.withdrawer_ata_y.to_account_info())
    };

    // Because transfer from vault
    let seeds: [&[&[u8]]; 1] = [&[
      b"config",
      &self.config.seed.to_le_bytes()[..],
      &[self.config.config_bump]
    ]];

    let cpi_program = self.token_program.to_account_info();
    let cpi_accs = Transfer{from, to, authority: self.withdrawer.to_account_info() };
    let ctx = CpiContext::new_with_signer(
      cpi_program,
      cpi_accs,
      &seeds
    );

    transfer(ctx, amount)
  }

  // 
  fn burn_lp_tokens(&self, lp_amount: u64) -> Result<()> {

    let cpi_program = self.token_program.to_account_info();

    let cpi_accs = Burn{
      mint: self.mint_lp.to_account_info(),
      from: self.withdrawer_ata_lp.to_account_info(),
      authority: self.withdrawer.to_account_info()
    };

    let ctx = CpiContext::new(cpi_program,cpi_accs);

    burn(ctx, lp_amount)
  }
}
