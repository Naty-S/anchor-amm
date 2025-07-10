use anchor_lang::prelude::*;

declare_id!("FN18FMQY7iizFvpQc2in2rwrwQRwshV4FcvnkaxDo1Wv");

#[program]
pub mod anchor_amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
