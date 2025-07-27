use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token::{ Mint, Token, TokenAccount, Transfer, MintTo, transfer, mint_to },
};
use constant_product_curve::ConstantProduct;

use crate::{error::AmmError, state::Config};


#[derive(Accounts)]
pub struct Deposit<'info> {

  #[account(mut)]
  pub depositer: Signer<'info>,
  
  pub mint_x: Account<'info, Mint>,  
  pub mint_y: Account<'info, Mint>,
  
  #[account(
    mut, // because changing the supply of the mint
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
    mut,
    associated_token::mint = mint_x,
    associated_token::authority = depositer,
  )]
  // From where to deposit x tokens to its vault
  pub depositer_ata_x: Account<'info, TokenAccount>,
  
  #[account(
    mut,
    associated_token::mint = mint_y,
    associated_token::authority = depositer,
  )]
  // From where to deposit y tokens to its vault
  pub depositer_ata_y: Account<'info, TokenAccount>,

  #[account(
    init_if_needed,
    payer = depositer,
    associated_token::mint = mint_lp,
    associated_token::authority = depositer,
  )]
  // To where mint the lp tokens wanted
  pub depositer_ata_lp: Account<'info, TokenAccount>,
  
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Deposit<'info> {
  
  // Deposit token pair to the pool
  pub fn deposit(
    &mut self,
    lp_amount: u64, // how many lp tokens depositer wants to get back (claim)
    
    // max tokens depositer is willing to deposit. To avoid slipage
    max_x: u64,
    max_y: u64,
  ) -> Result<()> {

    require!(self.config.locked == false, AmmError::PoolLocked);
    require!(lp_amount != 0, AmmError::InvalidAmount);

    // Calc how many 'x' and 'y' tokens user should deposit to get the lp tokens wanted
    let (x, y) = match
      // Initial state, there's no curve
      self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0
    {
      true => (max_x, max_y),
      // Someone already deposited, and need to adhere to the existing curve
      false => {
        // Calc the constant product curve and the respective x,y to deposit
        let amounts = ConstantProduct::xy_deposit_amounts_from_l(
          // current state of the pool
            self.vault_x.amount
          , self.vault_y.amount
          , self.mint_lp.supply
          , lp_amount
          , 6 // from lp decimanls
        ).unwrap();

        (amounts.x, amounts.y)
      }
    };

    require!(x <= max_x && y <= max_y, AmmError::SlippageExceeded);

    self.deposit_token(true, x)?;
    self.deposit_token(false, y)?;
    self.mint_lp_tokens(lp_amount)
  }

  // Deposit 'x' or 'y' tokens to the corresponding vault
  fn deposit_token(
    &self,
    is_x: bool,
    amount: u64,
  ) -> Result<()> {

    let (from, to) = match is_x {
      true => (self.depositer_ata_x.to_account_info(), self.vault_x.to_account_info()),
      false => (self.depositer_ata_y.to_account_info(), self.vault_y.to_account_info())
    };

    let cpi_program = self.token_program.to_account_info();
    let cpi_accs = Transfer{from, to, authority: self.depositer.to_account_info() };
    let ctx = CpiContext::new(cpi_program, cpi_accs);

    transfer(ctx, amount)
  }

  // 
  fn mint_lp_tokens(&self, lp_amount: u64) -> Result<()> {

    let cpi_accs = MintTo{
      mint: self.mint_lp.to_account_info(),
      to: self.depositer_ata_lp.to_account_info(),
      authority: self.config.to_account_info()
    };

    // by myself
    let signer_seeds: [&[&[u8]]; 1] = [&[
      b"config",
      &self.config.seed.to_le_bytes()[..],
      &[self.config.config_bump]
    ]];

    // from vid
    let seeds_other_way = &[
      &b"config"[..],
      &self.config.seed.to_le_bytes(),
      &[self.config.config_bump]
    ];
    let signer_seeds_other_way = &[&seeds_other_way[..]];

    let ctx = CpiContext::new_with_signer(
      self.token_program.to_account_info(),
      cpi_accs,
      &signer_seeds
    );

    mint_to(ctx, lp_amount)
  }
}
