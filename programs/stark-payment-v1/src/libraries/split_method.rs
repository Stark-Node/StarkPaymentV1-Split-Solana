use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("BLAqFZF7PKcwPEKwsS2sXdzzjwxDCion3HkQtK54mgQS");

#[program]
pub mod stark_payment_v1 {
    use super::*;

    pub fn split(ctx: Context<Split>, amounts: Vec<u64>) -> Result<()> {
        let payer = &ctx.accounts.payer;
        let payer_token_account = &ctx.accounts.payer_token_account;
        let token_program = &ctx.accounts.token_program;

        require!(
            ctx.remaining_accounts.len() == amounts.len(),
            ErrorCode::LengthMismatch
        );

        for (i, receiver_account_info) in ctx.remaining_accounts.iter().enumerate() {
            let amount = amounts[i];

            let receiver_token_account: Account<TokenAccount> =
                Account::try_from(receiver_account_info)?;

            let cpi_ctx = CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: payer_token_account.to_account_info(),
                    to: receiver_token_account.to_account_info(),
                    authority: payer.to_account_info(),
                },
            );

            token::transfer(cpi_ctx, amount)?;
        }

        Ok(())
    }

    pub fn splitPercentage(ctx: Context<Split>, amount: u64, percentages: Vec<u64>) -> Result<()> {
        let payer = &ctx.accounts.payer;
        let payer_token_account = &ctx.accounts.payer_token_account;
        let token_program = &ctx.accounts.token_program;

        require!(
            ctx.remaining_accounts.len() == percentages.len(),
            ErrorCode::LengthMismatch
        );

        for (i, receiver_account_info) in ctx.remaining_accounts.iter().enumerate() {
            let amount = (amount * percentages[i]) / 100;

            let receiver_token_account: Account<TokenAccount> =
                Account::try_from(receiver_account_info)?;

            let cpi_ctx = CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: payer_token_account.to_account_info(),
                    to: receiver_token_account.to_account_info(),
                    authority: payer.to_account_info(),
                },
            );

            token::transfer(cpi_ctx, amount)?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Split<'info> {

    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, constraint = payer_token_account.owner == payer.key(), constraint = payer_token_account.mint == mint.key())]
    pub payer_token_account: Account<'info, TokenAccount>,
    pub mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}


#[error_code]
pub enum ErrorCode {
    #[msg("The number of receivers and amounts do not match")]
    LengthMismatch,
}